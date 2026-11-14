use napi::bindgen_prelude::{
    BigInt, Env, FromNapiValue, Function, JsObjectValue, JsValue, Object as JsObject, Result,
    Unknown, ValueType,
};
use napi::{Property, PropertyAttributes};

use crate::value::{function, host, proto, reflect};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatSlot {
    pub raw: f64,
}

fn invalid(message: impl Into<String>) -> napi::Error {
    napi::Error::from_reason(message.into())
}

pub(crate) fn bigint_to_f64(value: &BigInt) -> f64 {
    let mut result = 0.0;
    for (index, word) in value.words.iter().enumerate() {
        result += (*word as f64) * 2f64.powi((index * 64) as i32);
    }
    if value.sign_bit { -result } else { result }
}

fn input_to_f64(env: &Env, input: Unknown<'_>) -> Result<f64> {
    match input.get_type()? {
        ValueType::Object => input
            .coerce_to_object()?
            .unwrap::<FloatSlot>()
            .copied()
            .map(|slot| slot.raw)
            .map_err(|_| invalid("expected a Float instance")),
        ValueType::Number => input.coerce_to_number()?.get_double(),
        ValueType::BigInt => {
            let value = unsafe { BigInt::from_napi_value(env.raw(), input.raw())? };
            Ok(bigint_to_f64(&value))
        }
        _ => Err(invalid(
            "Float accepts only Float instances, numbers, or bigints",
        )),
    }
}

fn unwrap_float(this: &JsObject<'_>) -> Result<FloatSlot> {
    this.unwrap::<FloatSlot>()
        .copied()
        .map_err(|_| invalid("method called on an incompatible receiver; expected Float"))
}

fn spawn<'env>(
    env: &'env Env,
    prototype: JsObject<'env>,
    slot: FloatSlot,
) -> Result<JsObject<'env>> {
    let boxed = env.create_double(slot.raw)?.coerce_to_object()?;
    let mut instance = proto::set(env, boxed, prototype)?;
    // `size_hint` currently decrements external-memory accounting on finalization
    // without a matching increment for Object::wrap in napi-rs v3.14.
    instance.wrap(slot, None)?;
    Ok(instance)
}

pub fn register<'env, 'exports>(
    exports: &mut JsObject<'exports>,
    env: &'env Env,
    value: JsObject<'env>,
    _value_prototype: JsObject<'env>,
) -> Result<()> {
    let mut prototype = reflect::create(env, Some(host::number_prototype(env)?))?;
    prototype.define_properties(&[Property::new()
        .with_utf8_name("value")?
        .with_getter_closure(|_env, this| {
            let slot = unwrap_float(&this.object)?;
            Ok(slot.raw)
        })
        .with_property_attributes(PropertyAttributes::Configurable)])?;

    let value_of: Function<(), f64> = function::from_rust_fn(env, "valueOf", |ctx| {
        let this: JsObject = ctx.this()?;
        Ok(unwrap_float(&this)?.raw)
    })?;
    let to_string: Function<(), String> = function::from_rust_fn(env, "toString", |ctx| {
        let this: JsObject = ctx.this()?;
        Ok(unwrap_float(&this)?.raw.to_string())
    })?;
    prototype.set_named_property("valueOf", value_of)?;
    prototype.set_named_property("toString", to_string)?;
    proto::install_to_primitive(env, &mut prototype)?;
    proto::install_to_string_tag(env, &mut prototype, "Float")?;
    host::set_float_prototype(env, &prototype)?;

    let constructor = function::constructor_from_rust_fn(env, "Float", |ctx| {
        let receiver: JsObject = ctx.this()?;
        let target_prototype = proto::get_prototype(ctx.env, receiver)?;
        let raw = match function::optional_arg::<Unknown>(&ctx, 0)? {
            Some(value) => input_to_f64(ctx.env, value)?,
            None => 0.0,
        };
        Ok(spawn(ctx.env, target_prototype, FloatSlot { raw })?.raw())
    })?;
    let mut float = constructor;
    float = proto::set(env, float, value.clone())?;
    float.define_properties(&[Property::new()
        .with_utf8_name("prototype")?
        .with_value(&prototype)
        .with_property_attributes(PropertyAttributes::empty())])?;
    prototype.define_properties(&[Property::new()
        .with_utf8_name("constructor")?
        .with_value(&float)
        .with_property_attributes(
            PropertyAttributes::Writable | PropertyAttributes::Configurable,
        )])?;
    proto::define_has_instance::<FloatSlot>(env, &mut float)?;
    let constructor_alias = float.clone();
    float.set_named_property("new", constructor_alias)?;

    let of: Function<(Unknown,), napi::sys::napi_value> =
        function::from_rust_fn(env, "of", |ctx| {
            let raw = input_to_f64(ctx.env, ctx.get(0)?)?;
            Ok(spawn(ctx.env, host::float_prototype(ctx.env)?, FloatSlot { raw })?.raw())
        })?;
    let cast: Function<(Unknown,), f64> =
        function::from_rust_fn(env, "cast", |ctx| input_to_f64(ctx.env, ctx.get(0)?))?;
    let is_nan: Function<(Unknown,), bool> = function::from_rust_fn(env, "isNaN", |ctx| {
        Ok(input_to_f64(ctx.env, ctx.get(0)?)?.is_nan())
    })?;
    let is_finite: Function<(Unknown,), bool> = function::from_rust_fn(env, "isFinite", |ctx| {
        Ok(input_to_f64(ctx.env, ctx.get(0)?)?.is_finite())
    })?;
    float.set_named_property("of", of)?;
    float.set_named_property("cast", cast)?;
    float.set_named_property("isNaN", is_nan)?;
    float.set_named_property("isFinite", is_finite)?;
    float.set_named_property("NORMAL_CELL_BITS", crate::value::memory::NORMAL_CELL_BITS)?;

    exports.set_named_property("Float", &float)?;
    let mut global = env.get_global()?;
    global.set_named_property("Float", float)
}
