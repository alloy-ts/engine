pub mod big_integer;
pub mod big_num;
pub mod boolean;
pub mod float;
pub mod function;
pub(crate) mod host;
pub mod integer;
pub mod memory;
pub mod object;
pub mod primitive;
pub(crate) mod proto;
pub(crate) mod reflect;
pub mod slot;
pub mod small_number;
pub mod string;
pub mod trait_object;

pub use slot::{ExtendedValueSlot, ValueSlot};

#[cfg(target_pointer_width = "32")]
pub type RuntimeCell = memory::CompactCell;
#[cfg(not(target_pointer_width = "32"))]
pub type RuntimeCell = memory::ScalarCell;

use napi::bindgen_prelude::{
    Env, Function, JsObjectValue, JsValue, Object as JsObject, Result, Unknown,
};
use napi::{Property, PropertyAttributes, Status};

/// Initializes the abstract Value constructor and shared Value.prototype.
pub(crate) fn register<'env, 'exports>(
    exports: &mut JsObject<'exports>,
    env: &'env Env,
) -> Result<(JsObject<'env>, JsObject<'env>)> {
    let mut prototype = reflect::create(env, Some(host::object_prototype(env)?))?;

    let constructor: Function<(), Unknown> =
        function::from_rust_fn(env, "Value", |_ctx| -> Result<Unknown> {
            Err(napi::Error::new(
                Status::InvalidArg,
                "Value is abstract and cannot be instantiated directly",
            ))
        })?;
    let mut value = constructor.coerce_to_object()?;

    value = proto::set(env, value, host::object(env)?)?;
    value.define_properties(&[Property::new()
        .with_utf8_name("prototype")?
        .with_value(&prototype)
        .with_property_attributes(PropertyAttributes::empty())])?;
    prototype.define_properties(&[Property::new()
        .with_utf8_name("constructor")?
        .with_value(&value)
        .with_property_attributes(
            PropertyAttributes::Writable | PropertyAttributes::Configurable,
        )])?;

    proto::install_to_primitive(env, &mut prototype)?;
    proto::install_to_string_tag(env, &mut prototype, "Value")?;
    proto::define_has_instance_predicate(env, &mut value, |object| {
        object.unwrap::<integer::IntegerSlot>().is_ok()
            || big_integer::is_integer(object)
            || big_num::is_big_num(object)
            || object.unwrap::<float::FloatSlot>().is_ok()
            || object.unwrap::<small_number::SmallNumberSlot>().is_ok()
    })?;

    exports.set_named_property("Value", &value)?;
    env.get_global()?.set_named_property("Value", &value)?;
    Ok((value, prototype))
}

#[cfg(test)]
mod tests {
    use super::slot::{ExtendedValueSlot, ValueSlot};

    #[test]
    fn fixed_width_slots_have_no_storage_overhead() {
        assert_eq!(std::mem::size_of::<ValueSlot>(), 8);
        assert_eq!(std::mem::size_of::<ExtendedValueSlot>(), 16);
    }
}
