use std::sync::Arc;

use napi::bindgen_prelude::{
  Env, Function, FunctionCallContext, JsObjectValue, JsValuesTupleIntoVec,
  Object as JsObject, Result, ToNapiValue,
};

use super::{function, reflect};

/// Native state attached to a JavaScript object.
///
/// `T` may be a concrete Rust type or an unsized trait object such as
/// `dyn DisplayValue`. The `Arc` keeps the native implementation alive for as
/// long as the JavaScript wrapper is reachable.
pub struct TraitInstance<T: ?Sized + Send + Sync + 'static> {
  inner: Arc<T>,
}

impl<T: ?Sized + Send + Sync + 'static> TraitInstance<T> {
  pub fn new(inner: Arc<T>) -> Self {
    Self { inner }
  }

  pub fn inner(&self) -> &T {
    self.inner.as_ref()
  }
}

/// Create a JavaScript instance with `prototype` as its real `[[Prototype]]`
/// and attach the native trait implementation to it.
pub fn create_trait_instance<'env, T>(
  env: &'env Env,
  prototype: JsObject<'env>,
  native: Arc<T>,
) -> Result<JsObject<'env>>
where
  T: ?Sized + Send + Sync + 'static,
{
  let mut instance = reflect::create(env, Some(prototype))?;
  instance.wrap(TraitInstance::new(native), None)?;
  Ok(instance)
}

/// Read the Rust trait implementation previously attached to a JS instance.
pub fn unwrap_trait<'a, T>(instance: &'a JsObject<'_>) -> Result<&'a T>
where
  T: ?Sized + Send + Sync + 'static,
{
  Ok(instance.unwrap::<TraitInstance<T>>()?.inner())
}

/// Create one shared JavaScript prototype method backed by a Rust callback.
///
/// The generated function obtains its receiver from `this`, unwraps the native
/// trait object, and dispatches through `invoke`. Install the resulting function
/// once on the trait prototype so all instances share the same JS function.
pub fn trait_method<'env, T, Args, Return, Invoke>(
  env: &'env Env,
  name: &str,
  invoke: Invoke,
) -> Result<Function<'env, Args, Return>>
where
  T: ?Sized + Send + Sync + 'static,
  Args: JsValuesTupleIntoVec,
  Return: ToNapiValue,
  Invoke: 'static + for<'call> Fn(&T, &FunctionCallContext<'call>) -> Result<Return>,
{
  function::from_rust_fn(env, name, move |ctx| {
    let this: JsObject = ctx.this()?;
    let native = unwrap_trait::<T>(&this)?;
    invoke(native, &ctx)
  })
}

/// Attach a Rust-backed trait method to a shared JavaScript prototype.
pub fn install_trait_method<'env, T, Args, Return, Invoke>(
  env: &'env Env,
  prototype: &mut JsObject<'env>,
  name: &str,
  invoke: Invoke,
) -> Result<()>
where
  T: ?Sized + Send + Sync + 'static,
  Args: JsValuesTupleIntoVec,
  Return: ToNapiValue,
  Invoke: 'static + for<'call> Fn(&T, &FunctionCallContext<'call>) -> Result<Return>,
{
  let method = trait_method::<T, Args, Return, Invoke>(env, name, invoke)?;
  prototype.set_named_property(name, method)
}
