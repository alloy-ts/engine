use napi::bindgen_prelude::{
  FnArgs, FromNapiValue, Function, JsObjectValue, Object, ToNapiValue, Unknown,
};
use napi::{sys, Env, JsValue, Result};
use napi_derive::napi;

/// The global object (`globalThis`).
#[derive(Clone, Copy)]
pub struct JsGlobal<'env>(Object<'env>);

/// A timer handle, as returned by `setTimeout` / `setInterval`.
#[derive(Clone, Copy)]
pub struct JsTimeout<'env>(Object<'env>);

/// The `JSON` namespace object.
#[derive(Clone, Copy)]
pub struct JSON<'env>(Object<'env>);

impl FromNapiValue for JsGlobal<'_> {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    Ok(JsGlobal(Object::from_raw(env, napi_val)))
  }
}

impl FromNapiValue for JsTimeout<'_> {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    Ok(JsTimeout(Object::from_raw(env, napi_val)))
  }
}

impl FromNapiValue for JSON<'_> {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    Ok(JSON(Object::from_raw(env, napi_val)))
  }
}

// `ToNapiValue` is what lets these be handed back to JS (or passed as call
// arguments). napi's blanket `impl<T: JsValue> ToNapiValue for T` does not
// apply — see the module docs — so each type gets an explicit impl that just
// forwards the underlying `napi_value`.
impl ToNapiValue for JsGlobal<'_> {
  unsafe fn to_napi_value(_env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    Ok(val.0.raw())
  }
}

impl ToNapiValue for JsTimeout<'_> {
  unsafe fn to_napi_value(_env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    Ok(val.0.raw())
  }
}

impl ToNapiValue for JSON<'_> {
  unsafe fn to_napi_value(_env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    Ok(val.0.raw())
  }
}

impl<'env> JsGlobal<'env> {
  /// The `globalThis` object of `env`.
  pub fn current(env: &Env) -> Result<JsGlobal<'static>> {
    Ok(JsGlobal(env.get_global()?.coerce_to_object()?))
  }

  /// The underlying object, for anything this wrapper does not expose itself.
  pub fn object(&self) -> Object<'env> {
    self.0
  }

  /// Does the global object have this property?
  pub fn has(&self, name: &str) -> Result<bool> {
    self.0.has_named_property(name)
  }

  /// Read a property without validating its type.
  pub fn get<T: FromNapiValue>(&self, name: &str) -> Result<T> {
    self.0.get_named_property_unchecked(name)
  }

  /// The `JSON` namespace object.
  pub fn json(&self) -> Result<JSON<'env>> {
    self.get("JSON")
  }

  /// `setInterval(handler, interval)`.
  ///
  /// `this` is bound to the global object, so the call also works in hosts
  /// where these timers reject an unbound receiver.
  pub fn set_interval<'h>(
    &self,
    handler: Function<'h, (), Unknown<'h>>,
    interval: f64,
  ) -> Result<JsTimeout<'env>> {
    let set_interval: TimerFn<'env, 'h> = self.0.get_named_property_unchecked("setInterval")?;
    set_interval.apply(self.0, FnArgs {
      data: (handler, interval),
    })
  }

  /// `clearInterval(timer)`.
  pub fn clear_interval<'t>(&self, timer: JsTimeout<'t>) -> Result<()> {
    let clear_interval: Function<'env, JsTimeout<'t>, ()> =
      self.0.get_named_property_unchecked("clearInterval")?;
    clear_interval.apply(self.0, timer)
  }

  /// `setTimeout(handler, delay)`.
  pub fn set_timeout<'h>(
    &self,
    handler: Function<'h, (), Unknown<'h>>,
    delay: f64,
  ) -> Result<JsTimeout<'env>> {
    let set_timeout: TimerFn<'env, 'h> = self.0.get_named_property_unchecked("setTimeout")?;
    set_timeout.apply(self.0, FnArgs {
      data: (handler, delay),
    })
  }

  /// `clearTimeout(timer)`.
  pub fn clear_timeout<'t>(&self, timer: JsTimeout<'t>) -> Result<()> {
    let clear_timeout: Function<'env, JsTimeout<'t>, ()> =
      self.0.get_named_property_unchecked("clearTimeout")?;
    clear_timeout.apply(self.0, timer)
  }
}

/// `(handler, ms) -> timer`, the shape shared by `setTimeout` / `setInterval`.
type TimerFn<'env, 'h> =
  Function<'env, FnArgs<(Function<'h, (), Unknown<'h>>, f64)>, JsTimeout<'env>>;

impl<'env> JsTimeout<'env> {
  /// Hand the handle back to JS as an opaque value.
  pub fn to_unknown(&self) -> Unknown<'env> {
    self.0.to_unknown()
  }
}

impl<'env> JSON<'env> {
  /// `JSON.stringify(value)`.
  pub fn stringify<V: ToNapiValue>(&self, value: V) -> Result<String> {
    let stringify: Function<'env, V, String> = self.0.get_named_property_unchecked("stringify")?;
    stringify.apply(self.0, value)
  }
}

/// JS-visible handle for the global object.
///
/// JS: `const global = new Global();` — the environment is injected by NAPI, so
/// the constructor takes no JavaScript arguments. The class is stateless: every
/// method resolves `globalThis` again through the environment of the current
/// call, so nothing has to be kept alive between calls.
#[napi]
pub struct Global;

#[napi]
impl Global {
  #[napi(constructor)]
  pub fn new() -> Self {
    Global
  }

  /// `globalThis.setTimeout(handler, delay)` — returns the timer handle.
  #[napi]
  pub fn set_timeout(
    &self,
    env: &Env,
    handler: Function<(), Unknown>,
    delay: f64,
  ) -> Result<Unknown<'_>> {
    let timer = JsGlobal::current(env)?.set_timeout(handler, delay)?;
    Ok(timer.to_unknown())
  }

  /// `globalThis.setInterval(handler, interval)` — returns the timer handle.
  #[napi]
  pub fn set_interval(
    &self,
    env: &Env,
    handler: Function<(), Unknown>,
    interval: f64,
  ) -> Result<Unknown<'_>> {
    let timer = JsGlobal::current(env)?.set_interval(handler, interval)?;
    Ok(timer.to_unknown())
  }

  /// `globalThis.clearTimeout(timer)`.
  #[napi]
  pub fn clear_timeout(&self, env: &Env, timer: Unknown) -> Result<()> {
    let timer = JsTimeout::from_unknown(timer)?;
    JsGlobal::current(env)?.clear_timeout(timer)
  }

  /// `globalThis.clearInterval(timer)`.
  #[napi]
  pub fn clear_interval(&self, env: &Env, timer: Unknown) -> Result<()> {
    let timer = JsTimeout::from_unknown(timer)?;
    JsGlobal::current(env)?.clear_interval(timer)
  }

  /// `JSON.stringify(value)` through the global `JSON` object.
  #[napi]
  pub fn stringify(&self, env: &Env, value: Unknown) -> Result<String> {
    JsGlobal::current(env)?.json()?.stringify(value)
  }

  /// Does the global object have this property?
  #[napi]
  pub fn has(&self, env: &Env, name: String) -> Result<bool> {
    JsGlobal::current(env)?.has(&name)
  }
}
