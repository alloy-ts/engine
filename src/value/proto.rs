use napi::bindgen_prelude::{
    Env, FnArgs, Function, JsObjectValue, JsValue, Object as JsObject, Result, Unknown, ValueType,
};

use super::{function as native_function, host};

pub(crate) fn object(env: &Env) -> Result<JsObject<'_>> {
    host::object_prototype(env)
}

pub(crate) fn function(env: &Env) -> Result<JsObject<'_>> {
    host::function_prototype(env)
}

pub(crate) fn set<'value>(
    env: &Env,
    target: JsObject<'value>,
    parent: JsObject<'value>,
) -> Result<JsObject<'value>> {
    let reflect: JsObject<'static> = env.get_global()?.get_named_property("Reflect")?;
    let set_prototype_of: Function<'static, FnArgs<(JsObject<'value>, JsObject<'value>)>, bool> =
        reflect.get_named_property("setPrototypeOf")?;
    let set_succeeded = set_prototype_of.call((target, parent).into())?;
    if !set_succeeded {
        return Err(napi::Error::from_reason(
            "Reflect.setPrototypeOf returned false",
        ));
    }
    Ok(target)
}

pub(crate) fn get_prototype<'value>(
    env: &Env,
    target: JsObject<'value>,
) -> Result<JsObject<'value>> {
    let reflect: JsObject<'static> = env.get_global()?.get_named_property("Reflect")?;
    let get_prototype_of: Function<'static, FnArgs<(JsObject<'value>,)>, JsObject<'value>> =
        reflect.get_named_property("getPrototypeOf")?;
    get_prototype_of.call((target,).into())
}

pub(crate) fn define_symbol_data_property(
    env: &Env,
    target: JsObject<'_>,
    key: Unknown<'_>,
    value: Unknown<'_>,
    writable: bool,
    configurable: bool,
) -> Result<()> {
    let reflect = host::reflect(env)?;
    let define_property: Function<FnArgs<(JsObject<'_>, Unknown<'_>, JsObject<'_>)>, bool> =
        reflect.get_named_property("defineProperty")?;
    let mut descriptor = JsObject::new(env)?;
    descriptor.set_named_property("value", value)?;
    descriptor.set_named_property("writable", writable)?;
    descriptor.set_named_property("enumerable", false)?;
    descriptor.set_named_property("configurable", configurable)?;
    if define_property.call((target, key, descriptor).into())? {
        Ok(())
    } else {
        Err(napi::Error::from_reason(
            "Reflect.defineProperty returned false",
        ))
    }
}

pub(crate) fn install_to_primitive(env: &Env, prototype: &mut JsObject<'_>) -> Result<()> {
    let symbol = host::symbol(env)?;
    let key: Unknown = symbol.get_named_property("toPrimitive")?;
    let method: Function<(String,), Unknown> =
        native_function::from_rust_fn(env, "[Symbol.toPrimitive]", |ctx| {
            let receiver: Unknown = ctx.this()?;
            if receiver.get_type()? != ValueType::Object {
                return Err(napi::Error::from_reason(
                    "Symbol.toPrimitive requires an object receiver",
                ));
            }
            let receiver = receiver.coerce_to_object()?;
            let hint = ctx
                .get::<Option<String>>(0)?
                .unwrap_or_else(|| "default".to_owned());
            let name = if hint == "string" {
                "toString"
            } else {
                "valueOf"
            };
            let conversion: Function<(), Unknown> = receiver.get_named_property(name)?;
            conversion.apply(receiver, ())
        })?;
    define_symbol_data_property(env, *prototype, key, method.to_unknown(), true, true)
}

pub(crate) fn install_to_string_tag(
    env: &Env,
    prototype: &mut JsObject<'_>,
    tag: &str,
) -> Result<()> {
    let symbol = host::symbol(env)?;
    let key: Unknown = symbol.get_named_property("toStringTag")?;
    let value = env.create_string(tag)?.to_unknown();
    define_symbol_data_property(env, *prototype, key, value, false, true)
}

pub(crate) fn define_has_instance<T: 'static>(
    env: &Env,
    constructor: &mut JsObject<'_>,
) -> Result<()> {
    let symbol = host::symbol(env)?;
    let key: Unknown = symbol.get_named_property("hasInstance")?;
    let checker: Function<(Unknown,), bool> =
        native_function::from_rust_fn(env, "[Symbol.hasInstance]", |ctx| {
            let candidate: Unknown = ctx.get(0)?;
            if candidate.get_type()? != ValueType::Object {
                return Ok(false);
            }
            Ok(candidate.coerce_to_object()?.unwrap::<T>().is_ok())
        })?;
    define_symbol_data_property(env, *constructor, key, checker.to_unknown(), false, true)
}

pub(crate) fn define_has_instance_predicate<F>(
    env: &Env,
    constructor: &mut JsObject<'_>,
    predicate: F,
) -> Result<()>
where
    F: 'static + Fn(&JsObject<'_>) -> bool,
{
    let symbol = host::symbol(env)?;
    let key: Unknown = symbol.get_named_property("hasInstance")?;
    let checker: Function<(Unknown,), bool> =
        native_function::from_rust_fn(env, "[Symbol.hasInstance]", move |ctx| {
            let candidate: Unknown = ctx.get(0)?;
            if candidate.get_type()? != ValueType::Object {
                return Ok(false);
            }
            Ok(predicate(&candidate.coerce_to_object()?))
        })?;
    define_symbol_data_property(env, *constructor, key, checker.to_unknown(), false, true)
}
