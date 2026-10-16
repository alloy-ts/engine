//! NAPI bindings for QuickJS Atom.

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// NAPI JS wrapper around QuickJS Atom representation.
#[napi]
pub struct Atom {
  pub(crate) name: String,
}

#[napi]
impl Atom {
  /// Create a new Atom with the given name string.
  #[napi(constructor)]
  pub fn new(name: String) -> Self {
    Atom { name }
  }

  /// Get the string representation of this Atom.
  #[napi]
  pub fn to_string_val(&self) -> String {
    self.name.clone()
  }
}
