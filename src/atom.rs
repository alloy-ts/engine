//! NAPI bindings for QuickJS [`Atom`].

use napi::bindgen_prelude::*;
use napi::Error;
use napi_derive::napi;

use crate::context::{js_value_to_serde, JsContext};

/// NAPI wrapper for QuickJS [`Atom`].
#[napi(js_name = "Atom")]
#[derive(Clone)]
pub struct JsAtom {
  pub(crate) name: String,
  #[allow(dead_code)]
  pub(crate) value: serde_json::Value,
}

#[napi]
impl JsAtom {
  /// Create an atom from a string in the given context.
  #[napi(factory, js_name = "from_str")]
  pub fn from_str(ctx: &JsContext, name: String) -> Result<Self> {
    let value = ctx.inner.with(|c| {
      let a = rquickjs::Atom::from_str(c.clone(), &name).map_err(|e| Error::from_reason(e.to_string()))?;
      let val = a.to_js_string().map_err(|e| Error::from_reason(e.to_string()))?.into_value();
      js_value_to_serde(&c, val)
    })?;

    Ok(Self { name, value })
  }

  /// Convert the atom to a string representation.
  #[napi(js_name = "to_string")]
  pub fn to_string(&self) -> Result<String> {
    Ok(self.name.clone())
  }
}
