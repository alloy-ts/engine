use napi::bindgen_prelude::{
    Env, Error, FromNapiValue, JsObjectValue, JsValue, Object as JsObject, ObjectRef, Result,
    Status, Unknown, ValidateNapiValue,
};

/// Per-environment references to the host runtime's original intrinsics.
///
/// JavaScript handles are never shared between Node workers. Each environment
/// captures its own realm's values during addon initialization.
pub struct Host {
    object: ObjectRef,
    object_prototype: ObjectRef,
    function: ObjectRef,
    number_prototype: ObjectRef,
    bigint_prototype: ObjectRef,
    /// Private object holding the original `Reflect`/`Object` functions the
    /// addon calls after initialization. Capturing the `Reflect` namespace alone
    /// is not enough: user code can still replace its methods in place.
    intrinsics: ObjectRef,
    symbol: ObjectRef,
    integer_prototype: Option<ObjectRef>,
    float_prototype: Option<ObjectRef>,
    sml_num_prototype: Option<ObjectRef>,
}

impl Host {
    fn capture(env: &Env) -> Result<Self> {
        let global = env.get_global()?;
        // Built-in constructors are JavaScript functions, not plain objects.
        // Read them as Unknown and box to an object handle before caching refs.
        let object: JsObject = global
            .get_named_property::<Unknown>("Object")?
            .coerce_to_object()?;
        let object_prototype: JsObject = object.get_named_property("prototype")?;
        let function: JsObject = global
            .get_named_property::<Unknown>("Function")?
            .coerce_to_object()?;
        let number: JsObject = global
            .get_named_property::<Unknown>("Number")?
            .coerce_to_object()?;
        let number_prototype: JsObject = number.get_named_property("prototype")?;
        let bigint: JsObject = global
            .get_named_property::<Unknown>("BigInt")?
            .coerce_to_object()?;
        let bigint_prototype: JsObject = bigint.get_named_property("prototype")?;
        let reflect: JsObject = global
            .get_named_property::<Unknown>("Reflect")?
            .coerce_to_object()?;
        let mut intrinsics = JsObject::new(env)?;
        for name in [
            "getPrototypeOf",
            "setPrototypeOf",
            "defineProperty",
            "construct",
        ] {
            let function: Unknown = reflect.get_named_property(name)?;
            intrinsics.set_named_property(&format!("Reflect.{name}"), function)?;
        }
        for name in ["create", "assign"] {
            let function: Unknown = object.get_named_property(name)?;
            intrinsics.set_named_property(&format!("Object.{name}"), function)?;
        }
        let symbol: JsObject = global
            .get_named_property::<Unknown>("Symbol")?
            .coerce_to_object()?;

        Ok(Self {
            object: object.create_ref()?,
            object_prototype: object_prototype.create_ref()?,
            function: function.create_ref()?,
            number_prototype: number_prototype.create_ref()?,
            bigint_prototype: bigint_prototype.create_ref()?,
            intrinsics: intrinsics.create_ref()?,
            symbol: symbol.create_ref()?,
            integer_prototype: None,
            float_prototype: None,
            sml_num_prototype: None,
        })
    }

    fn release(self, env: &Env) {
        let _ = self.object.unref(env);
        let _ = self.object_prototype.unref(env);
        let _ = self.function.unref(env);
        let _ = self.number_prototype.unref(env);
        let _ = self.bigint_prototype.unref(env);
        let _ = self.intrinsics.unref(env);
        let _ = self.symbol.unref(env);
        if let Some(integer_prototype) = self.integer_prototype {
            let _ = integer_prototype.unref(env);
        }
        if let Some(float_prototype) = self.float_prototype {
            let _ = float_prototype.unref(env);
        }
        if let Some(sml_num_prototype) = self.sml_num_prototype {
            let _ = sml_num_prototype.unref(env);
        }
    }
}

pub(crate) fn install(env: &Env) -> Result<()> {
    if env.get_instance_data::<Host>()?.is_some() {
        return Ok(());
    }

    let host = Host::capture(env)?;
    env.set_instance_data(host, (), |ctx| ctx.value.release(&ctx.env))
}

fn get(env: &Env) -> Result<&'static mut Host> {
    env.get_instance_data::<Host>()?.ok_or_else(|| {
        Error::new(
            Status::GenericFailure,
            "host intrinsics have not been initialized",
        )
    })
}

pub(crate) fn object(env: &Env) -> Result<JsObject<'_>> {
    get(env)?.object.get_value(env)
}

pub(crate) fn object_prototype(env: &Env) -> Result<JsObject<'_>> {
    get(env)?.object_prototype.get_value(env)
}

pub(crate) fn function(env: &Env) -> Result<JsObject<'_>> {
    get(env)?.function.get_value(env)
}

pub(crate) fn number_prototype(env: &Env) -> Result<JsObject<'_>> {
    get(env)?.number_prototype.get_value(env)
}

pub(crate) fn bigint_prototype(env: &Env) -> Result<JsObject<'_>> {
    get(env)?.bigint_prototype.get_value(env)
}

/// Look up a captured intrinsic function such as `"Reflect.construct"` or
/// `"Object.create"` (see `Host::capture` for the list).
pub(crate) fn intrinsic<'env, T: FromNapiValue + ValidateNapiValue>(
    env: &'env Env,
    name: &str,
) -> Result<T> {
    get(env)?
        .intrinsics
        .get_value(env)?
        .get_named_property(name)
}

pub(crate) fn symbol(env: &Env) -> Result<JsObject<'_>> {
    get(env)?.symbol.get_value(env)
}

pub(crate) fn set_integer_prototype(env: &Env, prototype: &JsObject<'_>) -> Result<()> {
    let host = get(env)?;
    if host.integer_prototype.is_some() {
        return Err(Error::new(
            Status::GenericFailure,
            "Integer prototype has already been initialized",
        ));
    }
    host.integer_prototype = Some(prototype.create_ref()?);
    Ok(())
}

pub(crate) fn integer_prototype(env: &Env) -> Result<JsObject<'_>> {
    get(env)?
        .integer_prototype
        .as_ref()
        .ok_or_else(|| Error::new(Status::GenericFailure, "Integer is not initialized"))?
        .get_value(env)
}

pub(crate) fn set_float_prototype(env: &Env, prototype: &JsObject<'_>) -> Result<()> {
    let host = get(env)?;
    if host.float_prototype.is_some() {
        return Err(Error::new(
            Status::GenericFailure,
            "Float prototype has already been initialized",
        ));
    }
    host.float_prototype = Some(prototype.create_ref()?);
    Ok(())
}

pub(crate) fn float_prototype(env: &Env) -> Result<JsObject<'_>> {
    get(env)?
        .float_prototype
        .as_ref()
        .ok_or_else(|| Error::new(Status::GenericFailure, "Float is not initialized"))?
        .get_value(env)
}

pub(crate) fn set_sml_num_prototype(env: &Env, prototype: &JsObject<'_>) -> Result<()> {
    let host = get(env)?;
    if host.sml_num_prototype.is_some() {
        return Err(Error::new(
            Status::GenericFailure,
            "SmlNum prototype has already been initialized",
        ));
    }
    host.sml_num_prototype = Some(prototype.create_ref()?);
    Ok(())
}

pub(crate) fn sml_num_prototype(env: &Env) -> Result<JsObject<'_>> {
    get(env)?
        .sml_num_prototype
        .as_ref()
        .ok_or_else(|| Error::new(Status::GenericFailure, "SmlNum is not initialized"))?
        .get_value(env)
}
