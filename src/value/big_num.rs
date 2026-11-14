use napi::bindgen_prelude::{
    Buffer, Env, Function, JsObjectValue, JsValue, Object as JsObject, Result, Unknown,
};
use napi::{Property, PropertyAttributes, Status};

use super::{
    function, host,
    memory::{BigNumFormat, HeapChunk, HeapKind},
    proto, reflect,
};

struct BigNumData {
    chunk: HeapChunk,
}

fn invalid(message: impl Into<String>) -> napi::Error {
    napi::Error::new(Status::InvalidArg, message.into())
}

fn format_name(format: BigNumFormat) -> &'static str {
    match format {
        BigNumFormat::Float80 => "float80",
        BigNumFormat::Float128 => "float128",
        BigNumFormat::Decimal128 => "decimal128",
        BigNumFormat::DoubleDouble => "doubleDouble",
        BigNumFormat::Custom => "custom",
    }
}

fn format_from_name(name: &str) -> Result<BigNumFormat> {
    match name {
        "float80" => Ok(BigNumFormat::Float80),
        "float128" => Ok(BigNumFormat::Float128),
        "decimal128" => Ok(BigNumFormat::Decimal128),
        "doubleDouble" => Ok(BigNumFormat::DoubleDouble),
        "custom" => Ok(BigNumFormat::Custom),
        _ => Err(invalid("unknown BigNum format")),
    }
}

fn decode_hex(text: &str) -> Result<Vec<u8>> {
    let text = text.strip_prefix("0x").unwrap_or(text);
    if text.len() % 2 != 0 {
        return Err(invalid("hex input must contain complete bytes"));
    }
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let chars = std::str::from_utf8(pair).map_err(|_| invalid("invalid hex input"))?;
            u8::from_str_radix(chars, 16).map_err(|_| invalid("invalid hex input"))
        })
        .collect()
}

fn new_chunk(format: BigNumFormat, bytes: &[u8]) -> Result<HeapChunk> {
    let expected = match format {
        BigNumFormat::Float80 => Some(10),
        BigNumFormat::Float128 | BigNumFormat::Decimal128 | BigNumFormat::DoubleDouble => Some(16),
        BigNumFormat::Custom => None,
    };
    if expected.is_some_and(|size| size != bytes.len()) {
        return Err(invalid("invalid byte length for BigNum format"));
    }
    HeapChunk::new(HeapKind::BigNum, 0, format as u16, bytes)
        .map_err(|_| invalid("unable to allocate BigNum chunk"))
}

fn data<'a, 'env>(object: &'a JsObject<'env>) -> Result<&'a BigNumData> {
    object
        .unwrap::<BigNumData>()
        .map(|value| &*value)
        .map_err(|_| invalid("expected a BigNum instance"))
}

pub(crate) fn is_big_num(object: &JsObject<'_>) -> bool {
    object.unwrap::<BigNumData>().is_ok()
}

fn spawn<'env>(
    env: &'env Env,
    prototype: JsObject<'env>,
    chunk: HeapChunk,
) -> Result<JsObject<'env>> {
    let mut object = reflect::create(env, Some(prototype))?;
    object.wrap(BigNumData { chunk }, None)?;
    Ok(object)
}

pub(crate) fn register<'env, 'exports>(
    exports: &mut JsObject<'exports>,
    env: &'env Env,
) -> Result<()> {
    let mut prototype = reflect::create(env, Some(host::object_prototype(env)?))?;

    let to_hex: Function<(), String> = function::from_rust_fn(env, "toHex", |ctx| {
        let this: JsObject = ctx.this()?;
        Ok(data(&this)?
            .chunk
            .payload()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect())
    })?;
    let to_buffer: Function<(), Buffer> = function::from_rust_fn(env, "toBuffer", |ctx| {
        let this: JsObject = ctx.this()?;
        Ok(Buffer::from(data(&this)?.chunk.payload().to_vec()))
    })?;
    let to_string: Function<(), String> = function::from_rust_fn(env, "toString", |ctx| {
        let this: JsObject = ctx.this()?;
        let chunk = &data(&this)?.chunk;
        let format = match chunk.sub_kind() {
            1 => BigNumFormat::Float80,
            2 => BigNumFormat::Float128,
            3 => BigNumFormat::Decimal128,
            4 => BigNumFormat::DoubleDouble,
            _ => BigNumFormat::Custom,
        };
        let hex: String = chunk
            .payload()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        Ok(format!("BigNum.{}(0x{hex})", format_name(format)))
    })?;
    prototype.set_named_property("toHex", to_hex)?;
    prototype.set_named_property("toBuffer", to_buffer)?;
    prototype.set_named_property("toString", to_string)?;
    prototype.define_properties(&[
        Property::new()
            .with_utf8_name("byteLength")?
            .with_getter_closure(|_env, this| Ok(data(&this.object)?.chunk.payload().len() as u32))
            .with_property_attributes(PropertyAttributes::Configurable),
        Property::new()
            .with_utf8_name("format")?
            .with_getter_closure(|_env, this| {
                let code = data(&this.object)?.chunk.sub_kind();
                Ok(format_name(match code {
                    1 => BigNumFormat::Float80,
                    2 => BigNumFormat::Float128,
                    3 => BigNumFormat::Decimal128,
                    4 => BigNumFormat::DoubleDouble,
                    _ => BigNumFormat::Custom,
                })
                .to_string())
            })
            .with_property_attributes(PropertyAttributes::Configurable),
    ])?;
    proto::install_to_string_tag(env, &mut prototype, "BigNum")?;

    let symbol = host::symbol(env)?;
    let primitive_key: Unknown = symbol.get_named_property("toPrimitive")?;
    let primitive: Function<(String,), napi::sys::napi_value> =
        function::from_rust_fn(env, "[Symbol.toPrimitive]", |ctx| {
            let this: JsObject = ctx.this()?;
            let chunk = &data(&this)?.chunk;
            let hint: String = ctx.get(0)?;
            if hint != "string" {
                return Err(invalid("BigNum requires explicit numeric conversion"));
            }
            let label = match chunk.sub_kind() {
                1 => "float80",
                2 => "float128",
                3 => "decimal128",
                4 => "doubleDouble",
                _ => "custom",
            };
            let hex: String = chunk
                .payload()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            Ok(ctx
                .env
                .create_string(&format!("BigNum.{label}(0x{hex})"))?
                .raw())
        })?;
    proto::define_symbol_data_property(
        env,
        prototype,
        primitive_key,
        primitive.to_unknown(),
        true,
        true,
    )?;

    let constructor: Function<(String, String), napi::sys::napi_value> =
        function::from_rust_fn(env, "BigNum", |ctx| {
            let target: JsObject = ctx.this()?;
            let prototype = proto::get_prototype(ctx.env, target)?;
            let format = format_from_name(&ctx.get::<String>(0)?)?;
            let bytes = decode_hex(&ctx.get::<String>(1)?)?;
            Ok(spawn(ctx.env, prototype, new_chunk(format, &bytes)?)?.raw())
        })?;
    let mut big_num = constructor.coerce_to_object()?;
    big_num.define_properties(&[Property::new()
        .with_utf8_name("prototype")?
        .with_value(&prototype)
        .with_property_attributes(PropertyAttributes::empty())])?;
    prototype.define_properties(&[Property::new()
        .with_utf8_name("constructor")?
        .with_value(&big_num)
        .with_property_attributes(
            PropertyAttributes::Writable | PropertyAttributes::Configurable,
        )])?;
    proto::define_has_instance::<BigNumData>(env, &mut big_num)?;
    let make: Function<(String, String), napi::sys::napi_value> =
        function::from_rust_fn(env, "fromHex", |ctx| {
            let format = format_from_name(&ctx.get::<String>(0)?)?;
            let bytes = decode_hex(&ctx.get::<String>(1)?)?;
            let constructor: JsObject = ctx.this()?;
            let prototype: JsObject = constructor.get_named_property("prototype")?;
            Ok(spawn(ctx.env, prototype, new_chunk(format, &bytes)?)?.raw())
        })?;
    big_num.set_named_property("fromHex", make)?;
    for format in [
        BigNumFormat::Float80,
        BigNumFormat::Float128,
        BigNumFormat::Decimal128,
        BigNumFormat::DoubleDouble,
    ] {
        let factory: Function<(String,), napi::sys::napi_value> =
            function::from_rust_fn(env, format_name(format), move |ctx| {
                let bytes = decode_hex(&ctx.get::<String>(0)?)?;
                let receiver: JsObject = ctx.this()?;
                let prototype: JsObject = receiver.get_named_property("prototype")?;
                Ok(spawn(ctx.env, prototype, new_chunk(format, &bytes)?)?.raw())
            })?;
        big_num.set_named_property(format_name(format), factory)?;
    }
    exports.set_named_property("BigNum", &big_num)?;
    env.get_global()?.set_named_property("BigNum", big_num)
}
