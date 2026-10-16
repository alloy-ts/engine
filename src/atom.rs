//! NAPI bindings for QuickJS Atom.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::Atom as QuickJSAtom;

use crate::context::Context;

/// NAPI wrapper around QuickJS Atom.
#[napi]
pub struct Atom {
  pub(crate) value: String,
}

#[napi]
impl Atom {
  /// Create an Atom from a string in a given Context.
  #[napi(factory, js_name = "from_string")]
  pub fn from_string(ctx: &Context, name: String) -> Result<Self> {
    ctx.inner.with(|c| {
      let atom = QuickJSAtom::from_str(c.clone(), &name)
        .map_err(|e| Error::from_reason(e.to_string()))?;
      let val_str = atom.to_string().map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { value: val_str })
    })
  }

  /// Create an Atom from a u32 index in a given Context.
  #[napi(factory, js_name = "from_u32")]
  pub fn from_u32(ctx: &Context, val: u32) -> Result<Self> {
    ctx.inner.with(|c| {
      let atom = QuickJSAtom::from_u32(c.clone(), val)
        .map_err(|e| Error::from_reason(e.to_string()))?;
      let val_str = atom.to_string().map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { value: val_str })
    })
  }

  /// Get string representation of the Atom.
  #[napi(js_name = "to_string")]
  pub fn to_string(&self) -> String {
    self.value.clone()
  }

  /// Convert the Atom to string (JS toString).
  #[napi(js_name = "toString")]
  pub fn js_to_string(&self) -> String {
    self.value.clone()
  }
}
