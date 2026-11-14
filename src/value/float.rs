use napi::bindgen_prelude::{
    BigInt, Env, FnArgs, FromNapiValue, Function, JsObjectValue, JsValue, Object as JsObject,
    Result, Unknown, ValueType,
};
use napi::{Property, PropertyAttributes};
use num_bigint::BigUint;
use num_traits::ToPrimitive;

use crate::value::{function, host, proto, reflect};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatSlot {
    pub raw: f64,
}

fn invalid(message: impl Into<String>) -> napi::Error {
    napi::Error::from_reason(message.into())
}

/// ECMAScript `Number(bigint)`: round to the nearest f64 (ties to even), and
/// overflow to an infinity. Summing per-word products double-rounds and turns
/// overflow into `NaN`, so delegate to num-bigint's correctly rounded
/// conversion instead.
pub(crate) fn bigint_to_f64(value: &BigInt) -> f64 {
    let magnitude = BigUint::new(
        value
            .words
            .iter()
            .flat_map(|word| [*word as u32, (*word >> 32) as u32])
            .collect(),
    );
    let result = magnitude.to_f64().unwrap_or(f64::INFINITY);
    // BigInt has no negative zero, so a zero magnitude stays +0.
    if value.sign_bit && result != 0.0 {
        -result
    } else {
        result
    }
}

/// `Number.prototype.toString(radix)` for a Number-family wrapper payload.
///
/// Without a radix this is ECMAScript `Number::toString` via
/// `napi_coerce_to_string` (`"Infinity"`, `"0"` for -0, `"1e+21"`), not Rust's
/// `f64` formatting. An explicit radix delegates to the captured host
/// `Number.prototype`, which also performs the spec's RangeError check.
pub(crate) fn number_to_string(env: &Env, raw: f64, radix: Option<Unknown<'_>>) -> Result<String> {
    let number = env.create_double(raw)?;
    match radix {
        None => number.coerce_to_string()?.into_utf8()?.into_owned(),
        Some(radix) => {
            let to_string: Function<FnArgs<(Unknown,)>, String> =
                host::number_prototype(env)?.get_named_property("toString")?;
            to_string.apply(number, (radix,).into())
        }
    }
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
            // SAFETY: `input` belongs to this callback's live `env` and is still
            // in scope, and this match arm has just checked
            // `get_type() == ValueType::BigInt`, so `napi_get_value_bigint_words`
            // receives a valid BigInt handle.
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
        let raw = unwrap_float(&this)?.raw;
        number_to_string(ctx.env, raw, function::optional_arg::<Unknown>(&ctx, 0)?)
    })?;
    prototype.set_named_property("valueOf", value_of)?;
    prototype.set_named_property("toString", to_string)?;
    proto::install_to_primitive(env, &mut prototype)?;
    proto::install_to_string_tag(env, &mut prototype, "Float")?;
    host::set_float_prototype(env, &prototype)?;

    let constructor = function::constructor_from_rust_fn(env, "Float", |ctx| {
        let receiver: JsObject = ctx.this()?;
        let target_prototype = proto::get_prototype(ctx.env, receiver)?;
        let raw = match function::optional_input(&ctx, 0)? {
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

#[cfg(test)]
mod tests {
    use super::{BigInt, bigint_to_f64};
    use num_bigint::BigUint;

    fn bigint(sign_bit: bool, decimal: &str) -> BigInt {
        let magnitude: BigUint = decimal.parse().unwrap();
        BigInt {
            sign_bit,
            words: magnitude.to_u64_digits(),
        }
    }

    #[test]
    fn bigint_to_f64_rounds_like_ecmascript_number() {
        // Rust's decimal parser is correctly rounded, matching `Number(bigint)`.
        for decimal in [
            "0",
            "9007199254740993",
            "18446744073709553665",
            "248061641579783547148052893441146846",
            "752154885170824791502373561466969566",
        ] {
            let expected: f64 = decimal.parse().unwrap();
            assert_eq!(
                bigint_to_f64(&bigint(false, decimal)),
                expected,
                "{decimal}"
            );
            assert_eq!(
                bigint_to_f64(&bigint(true, decimal)),
                -expected,
                "-{decimal}"
            );
        }
    }

    #[test]
    fn bigint_to_f64_overflows_to_infinity_and_has_no_negative_zero() {
        let huge = BigUint::from(1u8) << 1100u32;
        let words = huge.to_u64_digits();
        let positive = BigInt {
            sign_bit: false,
            words: words.clone(),
        };
        let negative = BigInt {
            sign_bit: true,
            words,
        };
        assert_eq!(bigint_to_f64(&positive), f64::INFINITY);
        assert_eq!(bigint_to_f64(&negative), f64::NEG_INFINITY);

        let zero = bigint_to_f64(&BigInt {
            sign_bit: true,
            words: vec![0],
        });
        assert_eq!(zero.to_bits(), 0.0f64.to_bits());
    }
}
