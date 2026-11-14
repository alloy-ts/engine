use napi::bindgen_prelude::{
    Env, FnArgs, FromNapiValue, Function, FunctionCallContext, JsObjectValue, JsValue,
    JsValuesTupleIntoVec, Object as JsObject, Result, ToNapiValue, Unknown,
};
use napi_derive::napi;

use super::{host, proto};

/// The host runtime's built-in `Function` constructor.
pub(crate) fn builtin_function(env: &Env) -> Result<JsObject<'_>> {
    host::function(env)
}

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
    let host_function: Function<FnArgs<(String, String)>, Unknown> =
        unsafe { Function::from_napi_value(env.raw(), host::function(env)?.raw())? };
    let factory_body = r#"
    const holder = { [name]: function () {
      "use strict";
      const target = new.target || (this === holder[name] ? holder[name] : undefined);
      if (!target) throw new TypeError("Constructor " + name + " requires 'new'");
      return reflect.construct(bridge, arguments, target);
    } };
    return holder[name];
  "#
    .to_owned();
    let factory_value = host_function.call(
        FnArgs {
            data: ("bridge, reflect, name".to_owned(), factory_body),
        }
        .into(),
    )?;
    let factory: Function<FnArgs<(Unknown, Unknown, String)>, Unknown> =
        unsafe { Function::from_napi_value(env.raw(), factory_value.raw())? };
    let constructor = factory.call(
        FnArgs {
            data: (
                bridge.to_unknown(),
                host::reflect(env)?.to_unknown(),
                name.to_owned(),
            ),
        }
        .into(),
    )?;
    constructor.coerce_to_object()
}

/// The prototype inherited by JavaScript functions.
pub(crate) fn function_prototype(env: &Env) -> Result<JsObject<'_>> {
    proto::function(env)
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
