use napi::bindgen_prelude::{
    Env, FnArgs, FromNapiValue, Function, FunctionCallContext, JsObjectValue, JsValue,
    JsValuesTupleIntoVec, Object as JsObject, Result, ToNapiValue, Unknown, ValueType,
};
use napi_derive::napi;

use super::{host, proto};

/// Turn a Rust callback into a JavaScript function object.
///
/// The callback is retained by NAPI-RS and invoked on the JavaScript thread.
/// This is intentionally an internal generic API: arbitrary Rust closures
/// cannot be passed into the addon from JavaScript as Rust function values.
pub(crate) fn from_rust_fn<'env, Args, Return, F>(
    env: &'env Env,
    name: &str,
    callback: F,
) -> Result<Function<'env, Args, Return>>
where
    Args: JsValuesTupleIntoVec,
    Return: ToNapiValue,
    F: 'static + Fn(FunctionCallContext) -> Result<Return>,
{
    env.create_function_from_closure(name, callback)
}

/// Read an optional argument, treating an omitted position like `undefined`.
/// `FunctionCallContext::get` rejects indices past `length()` before applying
/// napi-rs's `Option<T>` conversion, so defaulted parameters need this guard.
pub(crate) fn optional_arg<T: FromNapiValue>(
    ctx: &FunctionCallContext<'_>,
    index: usize,
) -> Result<Option<T>> {
    if index < ctx.length() {
        ctx.get::<Option<T>>(index)
    } else {
        Ok(None)
    }
}

/// Read an optional constructor *value* argument.
///
/// Omitted and `undefined` select the default (default-parameter semantics),
/// but `null` is returned as a value so the strict input parser rejects it.
/// `optional_arg::<Unknown>` cannot be used here: napi-rs's `Option<T>`
/// conversion maps `null` to `None`, silently turning `new Float(null)` into 0.
pub(crate) fn optional_input<'call>(
    ctx: &FunctionCallContext<'call>,
    index: usize,
) -> Result<Option<Unknown<'call>>> {
    if index >= ctx.length() {
        return Ok(None);
    }
    let value: Unknown<'call> = ctx.get(index)?;
    if value.get_type()? == ValueType::Undefined {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

/// Build a real JavaScript constructor around a Rust callback.
///
/// NAPI-RS closure callbacks do not expose `new.target` through
/// `FunctionCallContext`, so the outer JS function enforces `new` and passes
/// the actual `new.target` to the native bridge with `Reflect.construct`.
pub(crate) fn constructor_from_rust_fn<'env, Return, F>(
    env: &'env Env,
    name: &str,
    callback: F,
) -> Result<JsObject<'env>>
where
    Return: ToNapiValue,
    F: 'static + Fn(FunctionCallContext) -> Result<Return>,
{
    let bridge: Function<FnArgs<(Unknown, Unknown, Unknown)>, Return> =
        from_rust_fn(env, name, callback)?;
    // SAFETY: `host::function` returns a handle to the host `Function`
    // constructor captured at module init (a callable function object) and valid
    // in the current handle scope of `env`, which is the live environment.
    let host_function: Function<FnArgs<(String, String)>, Unknown> =
        unsafe { Function::from_napi_value(env.raw(), host::function(env)?.raw())? };
    let factory_body = r#"
    const holder = { [name]: function () {
      "use strict";
      const target = new.target || (this === holder[name] ? holder[name] : undefined);
      if (!target) throw new TypeError("Constructor " + name + " requires 'new'");
      return construct(bridge, arguments, target);
    } };
    return holder[name];
  "#
    .to_owned();
    let factory_value = host_function.call(
        FnArgs {
            data: ("bridge, construct, name".to_owned(), factory_body),
        }
        .into(),
    )?;
    // SAFETY: calling the `Function` constructor either throws (propagated by
    // `?` above) or returns a new function object; `factory_value` belongs to the
    // live `env` and the current handle scope.
    let factory: Function<FnArgs<(Unknown, Unknown, String)>, Unknown> =
        unsafe { Function::from_napi_value(env.raw(), factory_value.raw())? };
    let constructor = factory.call(
        FnArgs {
            data: (
                bridge.to_unknown(),
                host::intrinsic::<Unknown>(env, "Reflect.construct")?,
                name.to_owned(),
            ),
        }
        .into(),
    )?;
    constructor.coerce_to_object()
}

/// Set a constructor's prototype chain through `Reflect`.
pub(crate) fn extend_function<'value>(
    env: &Env,
    constructor: JsObject<'value>,
    parent: JsObject<'value>,
) -> Result<JsObject<'value>> {
    proto::set(env, constructor, parent)
}

/// Exposes the host Function prototype for later native bindings.
#[napi(js_name = "Function")]
pub mod function_api {
    use super::*;

    #[napi]
    pub fn prototype(env: Env) -> Result<JsObject<'static>> {
        let function: JsObject<'static> = env.get_global()?.get_named_property("Function")?;
        function.get_named_property("prototype")
    }

    #[napi(js_name = "setPrototypeOf")]
    pub fn set_prototype_of<'value>(
        env: Env,
        constructor: JsObject<'value>,
        parent: JsObject<'value>,
    ) -> Result<JsObject<'value>> {
        super::extend_function(&env, constructor, parent)
    }
}

#[allow(dead_code)]
fn _constructor_signature<'env>() -> Function<'env, (Unknown<'env>,), JsObject<'env>> {
    unreachable!("type-only signature")
}
