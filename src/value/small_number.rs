use napi::bindgen_prelude::{
    BigInt, Env, FromNapiValue, Function, JsObjectValue, JsValue, Object as JsObject, Result,
    Unknown, ValueType,
};
use napi::{Property, PropertyAttributes, Status};

use crate::value::{function, host, memory::SmlNum, proto, reflect};

pub type SmallNumberSlot = SmlNum;

fn invalid(message: impl Into<String>) -> napi::Error {
    napi::Error::new(Status::InvalidArg, message.into())
}

fn input_to_f64(env: &Env, input: Unknown<'_>) -> Result<f64> {
    match input.get_type()? {
        ValueType::Number => input.coerce_to_number()?.get_double(),
        ValueType::BigInt => {
            let value = unsafe { BigInt::from_napi_value(env.raw(), input.raw())? };
            Ok(crate::value::float::bigint_to_f64(&value))
        }
        ValueType::Object => {
            let object = input.coerce_to_object()?;
            if let Ok(slot) = object.unwrap::<SmallNumberSlot>() {
                return Ok(slot.to_f64());
            }
            if let Ok(slot) = object.unwrap::<crate::value::float::FloatSlot>() {
                return Ok(slot.raw);
            }
            Err(invalid(
                "SmlNum accepts only SmlNum, Float, Number, or BigInt values",
            ))
        }
        _ => Err(invalid("SmlNum accepts only Number or BigInt values")),
    }
}

fn unwrap_small_number(this: &JsObject<'_>) -> Result<SmallNumberSlot> {
    this.unwrap::<SmallNumberSlot>()
        .copied()
        .map_err(|_| invalid("method called on an incompatible receiver; expected SmlNum"))
}

fn quantize(value: f64) -> f64 {
    SmallNumberSlot::from_f64(value)
        .expect("numeric quantization is infallible")
        .to_f64()
}

fn spawn<'env>(env: &'env Env, prototype: JsObject<'env>, value: f64) -> Result<JsObject<'env>> {
    let slot = SmallNumberSlot::from_f64(value).expect("numeric quantization is infallible");
    let boxed = env.create_double(slot.to_f64())?.coerce_to_object()?;
    let mut instance = proto::set(env, boxed, prototype)?;
    instance.wrap(slot, None)?;
    Ok(instance)
}

pub fn register<'env, 'exports>(
    exports: &mut JsObject<'exports>,
    env: &'env Env,
    value: JsObject<'env>,
) -> Result<()> {
    let mut prototype = reflect::create(env, Some(host::number_prototype(env)?))?;
    prototype.define_properties(&[Property::new()
        .with_utf8_name("value")?
        .with_getter_closure(|_env, this| Ok(unwrap_small_number(&this.object)?.to_f64()))
        .with_property_attributes(PropertyAttributes::Configurable)])?;

    let value_of: Function<(), f64> = function::from_rust_fn(env, "valueOf", |ctx| {
        let this: JsObject = ctx.this()?;
        Ok(unwrap_small_number(&this)?.to_f64())
    })?;
    let to_string: Function<(), String> = function::from_rust_fn(env, "toString", |ctx| {
        let this: JsObject = ctx.this()?;
        Ok(unwrap_small_number(&this)?.to_f64().to_string())
    })?;
    prototype.set_named_property("valueOf", value_of)?;
    prototype.set_named_property("toString", to_string)?;
    proto::install_to_primitive(env, &mut prototype)?;
    proto::install_to_string_tag(env, &mut prototype, "SmlNum")?;
    host::set_sml_num_prototype(env, &prototype)?;

    let constructor = function::constructor_from_rust_fn(env, "SmlNum", |ctx| {
        let receiver: JsObject = ctx.this()?;
        let target_prototype = proto::get_prototype(ctx.env, receiver)?;
        let raw = match function::optional_arg::<Unknown>(&ctx, 0)? {
            Some(input) => input_to_f64(ctx.env, input)?,
            None => 0.0,
        };
        Ok(spawn(ctx.env, target_prototype, raw)?.raw())
    })?;
    let mut sml_num = constructor;
    sml_num = proto::set(env, sml_num, value.clone())?;
    sml_num.define_properties(&[Property::new()
        .with_utf8_name("prototype")?
        .with_value(&prototype)
        .with_property_attributes(PropertyAttributes::empty())])?;
    prototype.define_properties(&[Property::new()
        .with_utf8_name("constructor")?
        .with_value(&sml_num)
        .with_property_attributes(
            PropertyAttributes::Writable | PropertyAttributes::Configurable,
        )])?;
    proto::define_has_instance::<SmallNumberSlot>(env, &mut sml_num)?;
    let constructor_alias = sml_num.clone();
    sml_num.set_named_property("new", constructor_alias)?;

    let of: Function<(Unknown,), napi::sys::napi_value> =
        function::from_rust_fn(env, "of", |ctx| {
            let raw = input_to_f64(ctx.env, ctx.get(0)?)?;
            Ok(spawn(ctx.env, host::sml_num_prototype(ctx.env)?, raw)?.raw())
        })?;
    let cast: Function<(Unknown,), f64> = function::from_rust_fn(env, "cast", |ctx| {
        Ok(quantize(input_to_f64(ctx.env, ctx.get(0)?)?))
    })?;
    let is_nan: Function<(Unknown,), bool> = function::from_rust_fn(env, "isNaN", |ctx| {
        Ok(quantize(input_to_f64(ctx.env, ctx.get(0)?)?).is_nan())
    })?;
    let is_finite: Function<(Unknown,), bool> = function::from_rust_fn(env, "isFinite", |ctx| {
        Ok(quantize(input_to_f64(ctx.env, ctx.get(0)?)?).is_finite())
    })?;
    sml_num.set_named_property("of", of)?;
    sml_num.set_named_property("cast", cast)?;
    sml_num.set_named_property("isNaN", is_nan)?;
    sml_num.set_named_property("isFinite", is_finite)?;
    sml_num.set_named_property("BITS", crate::value::memory::SML_NUM_BITS)?;
    sml_num.set_named_property("NORMAL_CELL_BITS", crate::value::memory::NORMAL_CELL_BITS)?;

    exports.set_named_property("SmlNum", &sml_num)?;
    exports.set_named_property("SmlFlt", &sml_num)?;
    let mut global = env.get_global()?;
    global.set_named_property("SmlNum", &sml_num)?;
    global.set_named_property("SmlFlt", sml_num)
}
