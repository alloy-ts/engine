use napi::bindgen_prelude::{
  BigInt, Env, FromNapiValue, Function, JsObjectValue, JsValue, Object as JsObject,
  Result, This, ToNapiValue, Unknown, ValueType,
};
use napi::{Property, PropertyAttributes, Status};
use num_bigint::{BigInt as NumBigInt, Sign};
use num_traits::{Num, Signed, ToPrimitive, Zero};

use crate::value::{function, host, memory::SmlInt as CompactSmlInt, proto, reflect};

const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntegerSlot {
  pub compact: CompactSmlInt,
  pub bits: u32,
  pub unsigned: bool,
}

fn invalid(message: impl Into<String>) -> napi::Error {
  napi::Error::new(Status::InvalidArg, message.into())
}

fn validate_bits(bits: u32) -> Result<()> {
  if matches!(bits, 8 | 16 | 32 | 64 | 128) {
    Ok(())
  } else {
    Err(invalid(format!(
      "unsupported bit width {bits}; expected 8, 16, 32, 64, or 128"
    )))
  }
}

fn napi_bigint_to_num(value: &BigInt) -> NumBigInt {
  let mut digits = Vec::with_capacity(value.words.len() * 2);
  for word in &value.words {
    digits.push(*word as u32);
    digits.push((*word >> 32) as u32);
  }
  while digits.last() == Some(&0) {
    digits.pop();
  }
  if digits.is_empty() {
    return NumBigInt::zero();
  }
  NumBigInt::new(
    if value.sign_bit { Sign::Minus } else { Sign::Plus },
    digits,
  )
}

fn num_to_napi_bigint(value: &NumBigInt) -> BigInt {
  let (sign, digits) = value.to_u32_digits();
  let mut words = Vec::with_capacity(digits.len().div_ceil(2));
  for pair in digits.chunks(2) {
    let low = pair[0] as u64;
    let high = pair.get(1).copied().unwrap_or(0) as u64;
    words.push(low | (high << 32));
  }
  BigInt {
    sign_bit: sign == Sign::Minus,
    words,
  }
}

fn wrap_unsigned(value: &NumBigInt, bits: u32) -> NumBigInt {
  let modulus = NumBigInt::from(1u8) << bits;
  let mut value = value % &modulus;
  if value.is_negative() {
    value += modulus;
  }
  value
}

fn wrap_signed(value: &NumBigInt, bits: u32) -> NumBigInt {
  let modulus = NumBigInt::from(1u8) << bits;
  let sign = NumBigInt::from(1u8) << (bits - 1);
  let mut value = wrap_unsigned(value, bits);
  if value >= sign {
    value -= modulus;
  }
  value
}

fn normalize(value: &NumBigInt, bits: u32, unsigned: bool) -> Result<IntegerSlot> {
  validate_bits(bits)?;
  let normalized = if unsigned {
    wrap_unsigned(value, bits).to_u128().unwrap_or(0) as i128
  } else {
    wrap_signed(value, bits).to_i128().unwrap_or(0)
  };
  let compact = if unsigned {
    CompactSmlInt::new_unsigned(normalized as u64)
      .map_err(|_| invalid("normalized SmlInt exceeds the target-specific small integer width"))?
  } else {
    let raw = i32::try_from(normalized)
      .map_err(|_| invalid("normalized SmlInt does not fit its compact cell"))?;
    CompactSmlInt::new(raw)
      .map_err(|_| invalid("normalized SmlInt exceeds the target-specific small integer width"))?
  };
  Ok(IntegerSlot { compact, bits, unsigned })
}

fn slot_value(slot: IntegerSlot) -> NumBigInt {
  let raw = if slot.unsigned {
    NumBigInt::from(slot.compact.unsigned_value())
  } else {
    NumBigInt::from(slot.compact.signed_value())
  };
  if slot.unsigned {
    let mask = (NumBigInt::from(1u8) << slot.bits) - 1u8;
    raw & mask
  } else {
    wrap_signed(&raw, slot.bits)
  }
}

fn unwrap_integer(this: &JsObject<'_>) -> Result<IntegerSlot> {
  this
    .unwrap::<IntegerSlot>()
    .copied()
    .map_err(|_| invalid("method called on an incompatible receiver; expected SmlInt"))
}

fn strict_integer_input(env: &Env, input: Unknown<'_>) -> Result<NumBigInt> {
  match input.get_type()? {
    ValueType::Object => {
      let object = input.coerce_to_object()?;
      Ok(slot_value(unwrap_integer(&object)?))
    }
    ValueType::BigInt => {
      let value = unsafe { BigInt::from_napi_value(env.raw(), input.raw())? };
      Ok(napi_bigint_to_num(&value))
    }
    ValueType::Number => {
      let value = input.coerce_to_number()?.get_double()?;
      if !value.is_finite()
        || value.fract() != 0.0
        || value.abs() > MAX_SAFE_INTEGER as f64
      {
        return Err(invalid(
          "SmlInt accepts only finite, integral, safe-range JavaScript numbers",
        ));
      }
      Ok(NumBigInt::from(value as i64))
    }
    _ => Err(invalid(
      "SmlInt accepts only SmlInt instances, bigint values, or safe integers",
    )),
  }
}

fn validate_sml_int(value: &NumBigInt, unsigned: bool) -> Result<()> {
  let max = if unsigned {
    (NumBigInt::from(1u8) << crate::value::memory::SML_INT_BITS) - 1u8
  } else {
    NumBigInt::from(crate::value::memory::SML_INT_MAX)
  };
  if value < &NumBigInt::from(crate::value::memory::SML_INT_MIN) || value > &max {
    return Err(invalid(
      format!("SmlInt value does not fit the target {}-bit range", crate::value::memory::SML_INT_BITS),
    ));
  }
  Ok(())
}

fn slot_to_unknown<'env>(env: &'env Env, slot: IntegerSlot) -> Result<Unknown<'env>> {
  let value = slot_value(slot);
  if slot.bits <= 32 {
    if slot.unsigned {
      return Ok(env
        .create_uint32(value.to_u32().unwrap_or(0))?
        .to_unknown());
    }
    return Ok(env
      .create_int32(value.to_i32().unwrap_or(0))?
      .to_unknown());
  }
  num_to_napi_bigint(&value).into_unknown(env)
}

fn slot_to_number<'env>(env: &'env Env, slot: IntegerSlot) -> Result<Unknown<'env>> {
  let number = if slot.unsigned {
    slot.compact.unsigned_value() as f64
  } else {
    slot.compact.signed_value() as f64
  };
  Ok(env.create_double(number)?.to_unknown())
}

fn create_instance_with_prototype<'env>(
  env: &'env Env,
  prototype: JsObject<'env>,
  slot: IntegerSlot,
) -> Result<JsObject<'env>> {
  let number = slot_value(slot).to_f64().unwrap_or(0.0);
  let boxed = env.create_double(number)?.coerce_to_object()?;
  let mut instance = proto::set(env, boxed, prototype)?;
  // Keep wrapped slots unaccounted as external memory: napi-rs v3.14's
  // Object::wrap finalizer subtracts a size hint without adding it first.
  instance.wrap(slot, None)?;
  Ok(instance)
}

fn create_instance<'env>(env: &'env Env, slot: IntegerSlot) -> Result<JsObject<'env>> {
  create_instance_with_prototype(env, host::integer_prototype(env)?, slot)
}

fn parse_radix(input: &str, radix: u32) -> Result<NumBigInt> {
  if !(2..=36).contains(&radix) {
    return Err(invalid("radix must be between 2 and 36"));
  }
  NumBigInt::from_str_radix(input.trim(), radix)
    .map_err(|_| invalid(format!("cannot parse '{input}' as an Integer")))
}

fn install_instance_cast(
  env: &Env,
  prototype: &mut JsObject<'_>,
  name: &str,
  bits: u32,
) -> Result<()> {
  let method: Function<(Option<bool>,), napi::sys::napi_value> =
    function::from_rust_fn(env, name, move |ctx| {
      let this: JsObject = ctx.this()?;
      let slot = unwrap_integer(&this)?;
      let unsigned = !function::optional_arg::<bool>(&ctx, 0)?.unwrap_or(!slot.unsigned);
      Ok(slot_to_unknown(ctx.env, normalize(&slot_value(slot), bits, unsigned)?)?.raw())
    })?;
  prototype.set_named_property(name, method)
}

fn install_static_cast(
  env: &Env,
  integer: &mut JsObject<'_>,
  name: &str,
  bits: u32,
) -> Result<()> {
  let method: Function<(Unknown, Option<bool>), napi::sys::napi_value> =
    function::from_rust_fn(env, name, move |ctx| {
      let value = strict_integer_input(ctx.env, ctx.get(0)?)?;
      let signed = function::optional_arg::<bool>(&ctx, 1)?.unwrap_or(true);
      Ok(slot_to_unknown(ctx.env, normalize(&value, bits, !signed)?)?.raw())
    })?;
  integer.set_named_property(name, method)
}

pub fn register<'env, 'exports>(
  exports: &mut JsObject<'exports>,
  env: &'env Env,
  value: JsObject<'env>,
  _value_prototype: JsObject<'env>,
) -> Result<()> {
  let mut prototype = reflect::create(env, Some(host::number_prototype(env)?))?;

  prototype.define_properties(&[
    Property::new()
      .with_utf8_name("value")?
      .with_getter_closure(|_env, this: This| {
        Ok(slot_value(unwrap_integer(&this.object)?).to_f64().unwrap_or(0.0))
      })
      .with_property_attributes(PropertyAttributes::Configurable),
    Property::new()
      .with_utf8_name("bits")?
      .with_getter_closure(|_env, this: This| Ok(unwrap_integer(&this.object)?.bits))
      .with_property_attributes(PropertyAttributes::Configurable),
    Property::new()
      .with_utf8_name("bytes")?
      .with_getter_closure(|_env, this: This| Ok(unwrap_integer(&this.object)?.bits / 8))
      .with_property_attributes(PropertyAttributes::Configurable),
    Property::new()
      .with_utf8_name("unsigned")?
      .with_getter_closure(|_env, this: This| Ok(unwrap_integer(&this.object)?.unsigned))
      .with_property_attributes(PropertyAttributes::Configurable),
  ])?;

  let to_string: Function<(Option<u32>,), String> =
    function::from_rust_fn(env, "toString", |ctx| {
      let this: JsObject = ctx.this()?;
      let radix = function::optional_arg::<u32>(&ctx, 0)?.unwrap_or(10);
      if !(2..=36).contains(&radix) {
        return Err(invalid("toString radix must be between 2 and 36"));
      }
      Ok(slot_value(unwrap_integer(&this)?).to_str_radix(radix))
    })?;
  prototype.set_named_property("toString", to_string)?;

  let value_of: Function<(), napi::sys::napi_value> =
    function::from_rust_fn(env, "valueOf", |ctx| {
      let this: JsObject = ctx.this()?;
      Ok(slot_to_number(ctx.env, unwrap_integer(&this)?)?.raw())
    })?;
  prototype.set_named_property("valueOf", value_of)?;
  proto::install_to_primitive(env, &mut prototype)?;
  proto::install_to_string_tag(env, &mut prototype, "SmlInt")?;

  for (name, bits) in [
    ("asByte", 8),
    ("asI8", 8),
    ("asWord", 16),
    ("asI16", 16),
    ("asShort", 32),
    ("asI32", 32),
    ("asSmall", 32),
    ("asI64", 64),
    ("asLong", 128),
    ("asI128", 128),
    ("asBig", 128),
  ] {
    install_instance_cast(env, &mut prototype, name, bits)?;
  }

  let as_unsigned: Function<(Option<u32>,), napi::sys::napi_value> =
    function::from_rust_fn(env, "asUnsigned", |ctx| {
      let this: JsObject = ctx.this()?;
      let slot = unwrap_integer(&this)?;
      let bits = function::optional_arg::<u32>(&ctx, 0)?.unwrap_or(slot.bits);
      Ok(slot_to_unknown(ctx.env, normalize(&slot_value(slot), bits, true)?)?.raw())
    })?;
  prototype.set_named_property("asUnsigned", as_unsigned)?;

  install_instance_cast(env, &mut prototype, "asIsize", isize::BITS)?;

  host::set_integer_prototype(env, &prototype)?;

  let constructor = function::constructor_from_rust_fn(env, "SmlInt", |ctx| {
      let receiver: JsObject = ctx.this()?;
      let target_prototype = proto::get_prototype(ctx.env, receiver)?;

      let value = match function::optional_arg::<Unknown>(&ctx, 0)? {
        Some(value) => strict_integer_input(ctx.env, value)?,
        None => NumBigInt::zero(),
      };
      let bits = function::optional_arg::<u32>(&ctx, 1)?.unwrap_or(crate::value::memory::SML_INT_BITS);
      let unsigned = function::optional_arg::<bool>(&ctx, 2)?.unwrap_or(false);
      validate_sml_int(&value, unsigned)?;

      Ok(create_instance_with_prototype(
        ctx.env,
        target_prototype,
        normalize(&value, bits, unsigned)?,
      )?.raw())
    })?;
  let mut integer = constructor;

  integer = proto::set(env, integer, value)?;

  // Replace the function's default prototype value with the shared native
  // prototype. Keeping this as a data property preserves `new`/`instanceof`.
  integer.define_properties(&[
    Property::new()
      .with_utf8_name("prototype")?
      .with_value(&prototype)
      .with_property_attributes(PropertyAttributes::empty()),
  ])?;
  prototype.define_properties(&[
    Property::new()
      .with_utf8_name("constructor")?
      .with_value(&integer)
      .with_property_attributes(
        PropertyAttributes::Writable | PropertyAttributes::Configurable,
      ),
  ])?;
  proto::define_has_instance::<IntegerSlot>(env, &mut integer)?;
  let constructor_alias = integer.clone();
  integer.set_named_property("new", constructor_alias)?;

  let of: Function<(Unknown, Option<u32>, Option<bool>), napi::sys::napi_value> =
    function::from_rust_fn(env, "of", |ctx| {
      let value = strict_integer_input(ctx.env, ctx.get(0)?)?;
      let bits = function::optional_arg::<u32>(&ctx, 1)?.unwrap_or(crate::value::memory::SML_INT_BITS);
      let unsigned = function::optional_arg::<bool>(&ctx, 2)?.unwrap_or(false);
      validate_sml_int(&value, unsigned)?;
      Ok(create_instance(ctx.env, normalize(&value, bits, unsigned)?)?.raw())
    })?;

  let cast: Function<(Unknown, Option<u32>, Option<bool>), napi::sys::napi_value> =
    function::from_rust_fn(env, "cast", |ctx| {
      let value = strict_integer_input(ctx.env, ctx.get(0)?)?;
      let bits = function::optional_arg::<u32>(&ctx, 1)?.unwrap_or(crate::value::memory::SML_INT_BITS);
      let unsigned = function::optional_arg::<bool>(&ctx, 2)?.unwrap_or(false);
      validate_sml_int(&value, unsigned)?;
      Ok(slot_to_number(ctx.env, normalize(&value, bits, unsigned)?)?.raw())
    })?;

  let parse_int: Function<(String, Option<u32>, Option<u32>, Option<bool>), napi::sys::napi_value> =
    function::from_rust_fn(env, "parseInt", |ctx| {
      let input = ctx.get::<String>(0)?;
      let radix = function::optional_arg::<u32>(&ctx, 1)?.unwrap_or(10);
      let bits = function::optional_arg::<u32>(&ctx, 2)?.unwrap_or(crate::value::memory::SML_INT_BITS);
      let unsigned = function::optional_arg::<bool>(&ctx, 3)?.unwrap_or(false);
      let parsed = parse_radix(&input, radix)?;
      validate_sml_int(&parsed, unsigned)?;
      Ok(slot_to_number(
        ctx.env,
        normalize(&parsed, bits, unsigned)?,
      )?.raw())
    })?;

  let is_integer: Function<(Unknown,), bool> =
    function::from_rust_fn(env, "isInteger", |ctx| {
      let value: Unknown = ctx.get(0)?;
      match value.get_type()? {
        ValueType::Object => Ok(value
          .coerce_to_object()?
          .unwrap::<IntegerSlot>()
          .is_ok()),
        ValueType::Number => {
          let value = value.coerce_to_number()?.get_double()?;
          Ok(value.is_finite() && value.fract() == 0.0)
        }
        _ => Ok(false),
      }
    })?;

  let is_safe_integer: Function<(Unknown,), bool> =
    function::from_rust_fn(env, "isSafeInteger", |ctx| {
      let value: Unknown = ctx.get(0)?;
      let integer = match value.get_type()? {
        ValueType::Object => {
          let object = value.coerce_to_object()?;
          match object.unwrap::<IntegerSlot>() {
            Ok(slot) => slot_value(*slot),
            Err(_) => return Ok(false),
          }
        }
        ValueType::Number => {
          let value = value.coerce_to_number()?.get_double()?;
          return Ok(
            value.is_finite()
              && value.fract() == 0.0
              && value.abs() <= MAX_SAFE_INTEGER as f64,
          );
        }
        _ => return Ok(false),
      };
      Ok(integer.abs() <= NumBigInt::from(MAX_SAFE_INTEGER))
    })?;

  let as_int_n: Function<(u32, Unknown), BigInt> =
    function::from_rust_fn(env, "asIntN", |ctx| {
      let bits = ctx.get::<u32>(0)?;
      let value = strict_integer_input(ctx.env, ctx.get(1)?)?;
      if bits == 0 {
        return Ok(BigInt::from(0i64));
      }
      Ok(num_to_napi_bigint(&wrap_signed(&value, bits)))
    })?;

  let as_uint_n: Function<(u32, Unknown), BigInt> =
    function::from_rust_fn(env, "asUintN", |ctx| {
      let bits = ctx.get::<u32>(0)?;
      let value = strict_integer_input(ctx.env, ctx.get(1)?)?;
      if bits == 0 {
        return Ok(BigInt::from(0i64));
      }
      Ok(num_to_napi_bigint(&wrap_unsigned(&value, bits)))
    })?;

  integer.set_named_property("of", of)?;
  integer.set_named_property("cast", cast)?;
  integer.set_named_property("parseInt", parse_int)?;
  integer.set_named_property("isInteger", is_integer)?;
  integer.set_named_property("isSafeInteger", is_safe_integer)?;
  integer.set_named_property("asIntN", as_int_n)?;
  integer.set_named_property("asUintN", as_uint_n)?;

  for (name, bits) in [
    ("asByte", 8),
    ("asI8", 8),
    ("asWord", 16),
    ("asI16", 16),
    ("asShort", 32),
    ("asI32", 32),
    ("asSmall", 32),
    ("asI64", 64),
    ("asLong", 128),
    ("asI128", 128),
    ("asBig", 128),
    ("asIsize", isize::BITS),
  ] {
    install_static_cast(env, &mut integer, name, bits)?;
  }

  let static_as_unsigned: Function<(Unknown, Option<u32>), napi::sys::napi_value> =
    function::from_rust_fn(env, "asUnsigned", |ctx| {
      let value = strict_integer_input(ctx.env, ctx.get(0)?)?;
      let bits = function::optional_arg::<u32>(&ctx, 1)?.unwrap_or(crate::value::memory::SML_INT_BITS);
      Ok(slot_to_unknown(ctx.env, normalize(&value, bits, true)?)?.raw())
    })?;
  integer.set_named_property("asUnsigned", static_as_unsigned)?;
  integer.set_named_property("MAX_SAFE_INTEGER", MAX_SAFE_INTEGER as f64)?;
  integer.set_named_property("MIN_SAFE_INTEGER", -(MAX_SAFE_INTEGER as f64))?;
  integer.set_named_property("SML_BITS", crate::value::memory::SML_INT_BITS)?;
  integer.set_named_property("NORMAL_CELL_BITS", crate::value::memory::NORMAL_CELL_BITS)?;

  exports.set_named_property("SmlInt", &integer)?;
  let mut global = env.get_global()?;
  global.set_named_property("SmlInt", integer)
}
