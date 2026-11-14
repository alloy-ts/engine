//! NAPI facade over the runtime in [`crate::runtime`]. This is the only module
//! that converts between host `napi_value`s and runtime `Value`s.

use napi::JsString;
use napi::bindgen_prelude::{
    Either5, Env, Error, FromNapiValue, Function, JsObjectValue, JsValue, Null, Object, ObjectRef,
    Result, Status, Unknown, ValidateNapiValue, ValueType,
};
use napi_derive::napi;

use crate::runtime::{Completion, Disposed, ErrorKind, Exception, Realm, RuntimeContext, Value};

/// A runtime primitive copied out to the host.
type HostPrimitive = Either5<(), Null, bool, f64, String>;

const PRIMITIVE_TS: &str = "undefined | null | boolean | number | string";

/// Error constructors the facade throws, captured at addon load.
const ERROR_CONSTRUCTORS: [&str; 4] = ["SyntaxError", "ReferenceError", "TypeError", "RangeError"];

/// Per-environment references to the host's original error constructors, so
/// user patches of `globalThis.SyntaxError` etc. after load are ignored.
/// JavaScript handles are never shared between Node workers; each environment
/// captures its own during addon initialization.
struct HostIntrinsics {
    errors: ObjectRef,
}

#[napi(module_exports)]
pub fn init(_exports: Object, env: Env) -> Result<()> {
    if env.get_instance_data::<HostIntrinsics>()?.is_some() {
        return Ok(());
    }
    let global = env.get_global()?;
    let mut errors = Object::new(&env)?;
    for name in ERROR_CONSTRUCTORS {
        let constructor: Unknown = global.get_named_property(name)?;
        errors.set_named_property(name, constructor)?;
    }
    let intrinsics = HostIntrinsics {
        errors: errors.create_ref()?,
    };
    env.set_instance_data(intrinsics, (), |ctx| {
        let _ = ctx.value.errors.unref(&ctx.env);
    })
}

fn intrinsic<T: FromNapiValue + ValidateNapiValue>(env: &Env, name: &str) -> Result<T> {
    let intrinsics = env.get_instance_data::<HostIntrinsics>()?.ok_or_else(|| {
        Error::new(
            Status::GenericFailure,
            "host intrinsics have not been initialized",
        )
    })?;
    intrinsics.errors.get_value(env)?.get_named_property(name)
}

/// Creates isolated execution Contexts. Holds no shared runtime state.
#[napi]
pub struct Engine {}

#[napi]
impl Engine {
    #[napi(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    /// A new Context with its own Realm, globals and heap.
    #[napi]
    pub fn create_context(&self) -> Context {
        Context {
            runtime: RuntimeContext::new(),
        }
    }
}

/// A persistent handle to one Realm. Not an ECMAScript execution context.
/// Only `Engine.createContext()` creates one.
#[napi]
pub struct Context {
    runtime: RuntimeContext,
}

#[napi]
impl Context {
    /// Evaluates `source` as a Script and returns its completion value.
    #[napi(ts_return_type = "undefined | null | boolean | number | string")]
    pub fn eval(&mut self, env: &Env, source: JsString) -> Result<HostPrimitive> {
        let source = host_string(env, source)?;
        let realm = self.runtime.realm_mut().map_err(disposed)?;
        match realm.evaluate(&source) {
            Completion::Normal(value) => to_host(env, realm, value),
            Completion::Throw(value) => Err(throw_value(env, realm, value)),
        }
    }

    /// Reads a global binding the way a global identifier reference would.
    /// Throws ReferenceError when it does not exist or is in its temporal
    /// dead zone.
    #[napi(ts_return_type = "undefined | null | boolean | number | string")]
    pub fn get(&self, env: &Env, name: JsString) -> Result<HostPrimitive> {
        let name = host_string(env, name)?;
        let realm = self.runtime.realm().map_err(disposed)?;
        match realm.get_binding_value(&name) {
            Ok(value) => to_host(env, realm, value),
            Err(exception) => Err(throw(env, &exception)),
        }
    }

    /// Creates or updates a writable global binding from a host primitive.
    #[napi(ts_args_type = "name: string, value: undefined | null | boolean | number | string")]
    pub fn set(&mut self, env: &Env, name: JsString, value: Unknown) -> Result<()> {
        let name = host_string(env, name)?;
        // Check disposal before converting so a disposed context never
        // allocates.
        self.runtime.realm().map_err(disposed)?;
        let value = from_host(env, value)?;
        let realm = self.runtime.realm_mut().map_err(disposed)?;
        let value = match value {
            Imported::Value(value) => value,
            Imported::String(contents) => realm.alloc_string(contents),
        };
        realm
            .set_global(&name, value)
            .map_err(|exception| throw(env, &exception))
    }

    /// Releases the Realm. Idempotent; every later call except `dispose`
    /// and `disposed` throws.
    #[napi]
    pub fn dispose(&mut self) {
        self.runtime.dispose();
    }

    #[napi(getter)]
    pub fn disposed(&self) -> bool {
        self.runtime.is_disposed()
    }
}

fn disposed(_: Disposed) -> Error {
    Error::from_reason("Context is disposed")
}

/// Throws a host error of `exception.kind`, using the constructor captured at
/// addon load.
fn throw(env: &Env, exception: &Exception) -> Error {
    let thrown = (|| -> Result<()> {
        let constructor: Function<String, Unknown> = intrinsic(env, exception.kind.name())?;
        let error = constructor.new_instance(exception.message.clone())?;
        env.throw(error)
    })();
    match thrown {
        Ok(()) => Error::new(Status::PendingException, exception.message.clone()),
        Err(error) => error,
    }
}

fn throw_value(env: &Env, realm: &Realm, value: Value) -> Error {
    if let Value::Error(handle) = value {
        return throw(env, realm.heap().error(handle));
    }
    let thrown = match value {
        Value::Undefined => env.throw(()),
        Value::Null => env.throw(Null),
        Value::Boolean(value) => env.throw(value),
        Value::Number(value) => env.throw(value),
        Value::String(handle) => env.throw(realm.heap().string(handle).to_owned()),
        Value::Error(_) => unreachable!(),
    };
    match thrown {
        Ok(()) => Error::new(Status::PendingException, "script threw a value"),
        Err(error) => error,
    }
}

fn type_error(env: &Env, message: String) -> Error {
    throw(env, &Exception::new(ErrorKind::TypeError, message))
}

/// Host strings are UTF-16 and may hold lone surrogates, which the runtime's
/// UTF-8 strings cannot represent; reject them rather than replace them.
fn host_string(env: &Env, string: JsString) -> Result<String> {
    // `as_str` drops the NUL terminator napi-rs appends to the UTF-16 buffer
    // (`as_slice` includes it) and fails on unpaired surrogates.
    string.into_utf16()?.as_str().map_err(|_| {
        type_error(
            env,
            "strings containing lone surrogates are not supported".into(),
        )
    })
}

fn to_host(env: &Env, realm: &Realm, value: Value) -> Result<HostPrimitive> {
    Ok(match value {
        Value::Undefined => Either5::A(()),
        Value::Null => Either5::B(Null),
        Value::Boolean(b) => Either5::C(b),
        Value::Number(n) => Either5::D(n),
        Value::String(handle) => Either5::E(realm.heap().string(handle).to_owned()),
        Value::Error(_) => {
            return Err(type_error(
                env,
                "Cannot transfer a runtime error object; expected a primitive".into(),
            ));
        }
    })
}

enum Imported {
    Value(Value),
    String(String),
}

/// Checked conversion of a host value; anything but the supported primitives
/// is a TypeError. Coercions below only run on values whose type was just
/// checked, so they cannot invoke user code.
fn from_host(env: &Env, value: Unknown) -> Result<Imported> {
    Ok(match value.get_type()? {
        ValueType::Undefined => Imported::Value(Value::Undefined),
        ValueType::Null => Imported::Value(Value::Null),
        ValueType::Boolean => Imported::Value(Value::Boolean(value.coerce_to_bool()?)),
        ValueType::Number => {
            Imported::Value(Value::Number(value.coerce_to_number()?.get_double()?))
        }
        ValueType::String => Imported::String(host_string(env, value.coerce_to_string()?)?),
        other => {
            let kind = match other {
                ValueType::Object => "object",
                ValueType::Function => "function",
                ValueType::Symbol => "symbol",
                ValueType::BigInt => "bigint",
                ValueType::External => "external",
                _ => "unknown",
            };
            return Err(type_error(
                env,
                format!("Cannot transfer a host {kind}; expected {PRIMITIVE_TS}"),
            ));
        }
    })
}
