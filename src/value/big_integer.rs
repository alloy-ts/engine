use napi::bindgen_prelude::{
    BigInt, Env, FromNapiValue, Function, JsObjectValue, JsValue, Object as JsObject, Result,
    ToNapiValue, Unknown, ValueType,
};
use napi::{Property, PropertyAttributes, Status};
use num_bigint::{BigInt as NumBigInt, Sign};
use num_traits::{Signed, Zero};

use super::{function, host, proto, reflect};

const MAX_SAFE: i64 = 9_007_199_254_740_991;

struct IntegerData {
    value: NumBigInt,
    bits: Option<u32>,
    unsigned: bool,
}

fn invalid(message: impl Into<String>) -> napi::Error {
    napi::Error::new(Status::InvalidArg, message.into())
}

fn from_js(value: &BigInt) -> NumBigInt {
    let mut bytes = Vec::with_capacity(value.words.len() * 8);
    for word in &value.words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    NumBigInt::from_bytes_le(
        if value.sign_bit {
            Sign::Minus
        } else {
            Sign::Plus
        },
        &bytes,
    )
}

fn to_js(value: &NumBigInt) -> BigInt {
    let (sign, words) = value.to_u64_digits();
    BigInt {
        sign_bit: sign == Sign::Minus,
        words,
    }
}

fn input(env: &Env, value: Unknown<'_>) -> Result<NumBigInt> {
    match value.get_type()? {
        // SAFETY: `value` belongs to this callback's live `env` and is still
        // in scope, and this match arm has just checked
        // `get_type() == ValueType::BigInt`, so `napi_get_value_bigint_words`
        // receives a valid BigInt handle.
        ValueType::BigInt => Ok(from_js(&unsafe {
            BigInt::from_napi_value(env.raw(), value.raw())?
        })),
        ValueType::Number => {
            let number = value.coerce_to_number()?.get_double()?;
            if !number.is_finite() || number.fract() != 0.0 || number.abs() > MAX_SAFE as f64 {
                return Err(invalid(
                    "Integer requires a safe integral Number or a BigInt",
                ));
            }
            Ok(NumBigInt::from(number as i64))
        }
        ValueType::Object => Ok(data(&value.coerce_to_object()?)?.value.clone()),
        _ => Err(invalid(
            "Integer requires a BigInt, safe integer, or Integer instance",
        )),
    }
}

fn normalize(value: NumBigInt, bits: Option<u32>, unsigned: bool) -> NumBigInt {
    let Some(width) = bits else {
        return value;
    };
    if width == 0 {
        return NumBigInt::zero();
    }
    let modulus = NumBigInt::from(1u8) << width;
    let mut result = value % &modulus;
    if result.is_negative() {
        result += &modulus;
    }
    if !unsigned && result >= (NumBigInt::from(1u8) << (width - 1)) {
        result -= modulus;
    }
    result
}

fn data<'a, 'env>(object: &'a JsObject<'env>) -> Result<&'a IntegerData> {
    object
        .unwrap::<IntegerData>()
        .map(|value| &*value)
        .map_err(|_| invalid("expected an Integer instance"))
}

fn spawn<'env>(
    env: &'env Env,
    prototype: JsObject<'env>,
    value: NumBigInt,
    bits: Option<u32>,
    unsigned: bool,
) -> Result<JsObject<'env>> {
    let value = normalize(value, bits, unsigned);
    let boxed = to_js(&value).into_unknown(env)?.coerce_to_object()?;
    let mut object = proto::set(env, boxed, prototype)?;
    object.wrap(
        IntegerData {
            value,
            bits,
            unsigned,
        },
        None,
    )?;
    Ok(object)
}

pub(crate) fn register<'env, 'exports>(
    exports: &mut JsObject<'exports>,
    env: &'env Env,
) -> Result<()> {
    let mut prototype = reflect::create(env, Some(host::bigint_prototype(env)?))?;
    prototype.define_properties(&[
        Property::new()
            .with_utf8_name("value")?
            .with_getter_closure(|_env, this| Ok(to_js(&data(&this.object)?.value)))
            .with_property_attributes(PropertyAttributes::Configurable),
        Property::new()
            .with_utf8_name("bits")?
            .with_getter_closure(|_env, this| Ok(data(&this.object)?.bits))
            .with_property_attributes(PropertyAttributes::Configurable),
        Property::new()
            .with_utf8_name("unsigned")?
            .with_getter_closure(|_env, this| Ok(data(&this.object)?.unsigned))
            .with_property_attributes(PropertyAttributes::Configurable),
    ])?;
    proto::install_to_string_tag(env, &mut prototype, "Integer")?;

    let constructor = function::constructor_from_rust_fn(env, "Integer", |ctx| {
        let target: JsObject = ctx.this()?;
        let prototype = proto::get_prototype(ctx.env, target)?;
        let value = match function::optional_input(&ctx, 0)? {
            Some(arg) => input(ctx.env, arg)?,
            None => NumBigInt::zero(),
        };
        Ok(spawn(
            ctx.env,
            prototype,
            value,
            function::optional_arg::<u32>(&ctx, 1)?,
            function::optional_arg::<bool>(&ctx, 2)?.unwrap_or(false),
        )?
        .raw())
    })?;
    let mut integer = constructor;
    integer.define_properties(&[Property::new()
        .with_utf8_name("prototype")?
        .with_value(&prototype)
        .with_property_attributes(PropertyAttributes::empty())])?;
    prototype.define_properties(&[Property::new()
        .with_utf8_name("constructor")?
        .with_value(&integer)
        .with_property_attributes(
            PropertyAttributes::Writable | PropertyAttributes::Configurable,
        )])?;
    proto::define_has_instance::<IntegerData>(env, &mut integer)?;

    let of: Function<(Unknown, Option<u32>, Option<bool>), napi::sys::napi_value> =
        function::from_rust_fn(env, "of", |ctx| {
            let receiver: JsObject = ctx.this()?;
            let prototype: JsObject = receiver.get_named_property("prototype")?;
            let value = input(ctx.env, ctx.get(0)?)?;
            Ok(spawn(
                ctx.env,
                prototype,
                value,
                function::optional_arg::<u32>(&ctx, 1)?,
                function::optional_arg::<bool>(&ctx, 2)?.unwrap_or(false),
            )?
            .raw())
        })?;
    let cast: Function<(Unknown, Option<u32>, Option<bool>), BigInt> =
        function::from_rust_fn(env, "cast", |ctx| {
            let value = input(ctx.env, ctx.get(0)?)?;
            Ok(to_js(&normalize(
                value,
                function::optional_arg::<u32>(&ctx, 1)?,
                function::optional_arg::<bool>(&ctx, 2)?.unwrap_or(false),
            )))
        })?;
    let as_int_n: Function<(u32, Unknown), BigInt> =
        function::from_rust_fn(env, "asIntN", |ctx| {
            let bits = ctx.get(0)?;
            Ok(to_js(&normalize(
                input(ctx.env, ctx.get(1)?)?,
                Some(bits),
                false,
            )))
        })?;
    let as_uint_n: Function<(u32, Unknown), BigInt> =
        function::from_rust_fn(env, "asUintN", |ctx| {
            let bits = ctx.get(0)?;
            Ok(to_js(&normalize(
                input(ctx.env, ctx.get(1)?)?,
                Some(bits),
                true,
            )))
        })?;
    integer.set_named_property("of", of)?;
    integer.set_named_property("cast", cast)?;
    integer.set_named_property("asIntN", as_int_n)?;
    integer.set_named_property("asUintN", as_uint_n)?;
    integer.set_named_property("NORMAL_CELL_BITS", super::memory::NORMAL_CELL_BITS)?;
    exports.set_named_property("Integer", &integer)?;
    env.get_global()?.set_named_property("Integer", integer)
}

pub(crate) fn is_integer(object: &JsObject<'_>) -> bool {
    object.unwrap::<IntegerData>().is_ok()
}
