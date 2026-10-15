use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::Atom as Atom;

use crate::context::Context;

#[napi(object)]
pub struct Atom {
  pub(crate) value: String,
}

#[napi]
impl Atom {
  /// Create an Atom from a string in a given Context.
  #[napi(factory, js_name = "from_string")]
  pub fn from_string(ctx: &Context, name: String) -> Result<Self> {
    ctx.inner.with(|c| {
      let atom = Atom::from_str(c.clone(), &name)
        .map_err(|e| Error::from_reason(e.to_string()))?;
      let val_str = atom.to_string().map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { value: val_str })
    })
  }

  /// Create an Atom from a u32 integer in a given Context.
  #[napi(factory)]
  pub fn from_u32(ctx: &Context, val: u32) -> Result<Self> {
    ctx.inner.with(|c| {
      let atom = Atom::from_u32(c.clone(), val)
        .map_err(|e| Error::from_reason(e.to_string()))?;
      let val_str = atom.to_string().map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { value: val_str })
    })
  }

  /// Get string representation of the Atom.
  #[napi]
  pub fn to_string(&self) -> String {
    self.value.clone()
  }
}
