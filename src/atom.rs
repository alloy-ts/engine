//! NAPI bindings for QuickJS [`Atom`].

use napi::bindgen_prelude::*;
use napi::Error;
use napi_derive::napi;

use crate::context::JsContext;

/// NAPI wrapper for QuickJS [`Atom`].
#[napi(js_name = "Atom")]
#[derive(Clone)]
pub struct JsAtom {
  pub(crate) inner: rquickjs::Atom<'static>,
}

#[napi]
impl JsAtom {
  /// Create an atom from a string in the given context.
  #[napi(factory, js_name = "from_str")]
  pub fn from_str(ctx: &JsContext, name: String) -> Result<Self> {
    ctx.inner.with(|c| {
      let atom = rquickjs::Atom::from_str(c, &name).map_err(|e| Error::from_reason(e.to_string()))?;
      let static_atom =
        unsafe { std::mem::transmute::<rquickjs::Atom<'_>, rquickjs::Atom<'static>>(atom) };
      Ok(Self { inner: static_atom })
    })
  }

  /// Convert the atom to a string representation.
  #[napi(js_name = "to_string")]
  pub fn to_string(&self) -> Result<String> {
    self
      .inner
      .to_string()
      .map_err(|e| Error::from_reason(e.to_string()))
  }
}
