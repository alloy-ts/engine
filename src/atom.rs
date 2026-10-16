//! NAPI bindings for QuickJS Atom.

use napi_derive::napi;
use crate::context::Context;

/// JS-visible wrapper for QuickJS Atom.
#[napi(js_name = "Atom")]
pub struct Atom {
  pub(crate) value: String,
}

#[napi]
impl Atom {
  /// Create an atom from a string in the given Context.
  #[napi(constructor)]
  pub fn new(context: &Context, name: String) -> napi::Result<Self> {
    let value = context.inner.with(|ctx| {
      let atom = rquickjs::Atom::from_str(ctx, &name)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
      atom.to_string().map_err(|e| napi::Error::from_reason(e.to_string()))
    })?;
    Ok(Self { value })
  }

  /// Create an Atom from a u32 number in the given Context.
  #[napi]
  pub fn from_u32(context: &Context, val: u32) -> napi::Result<Self> {
    let value = context.inner.with(|ctx| {
      let atom = rquickjs::Atom::from_u32(ctx, val)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
      atom.to_string().map_err(|e| napi::Error::from_reason(e.to_string()))
    })?;
    Ok(Self { value })
  }

  /// Create an Atom from an i32 number in the given Context.
  #[napi]
  pub fn from_i32(context: &Context, val: i32) -> napi::Result<Self> {
    let value = context.inner.with(|ctx| {
      let atom = rquickjs::Atom::from_i32(ctx, val)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
      atom.to_string().map_err(|e| napi::Error::from_reason(e.to_string()))
    })?;
    Ok(Self { value })
  }

  /// Create an Atom from a boolean in the given Context.
  #[napi]
  pub fn from_bool(context: &Context, val: bool) -> napi::Result<Self> {
    let value = context.inner.with(|ctx| {
      let atom = rquickjs::Atom::from_bool(ctx, val)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
      atom.to_string().map_err(|e| napi::Error::from_reason(e.to_string()))
    })?;
    Ok(Self { value })
  }

  /// Convert atom to string representation.
  #[napi]
  pub fn to_string(&self) -> String {
    self.value.clone()
  }
}
