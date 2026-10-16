//! NAPI bindings for QuickJS / JS `Atom`.

use napi::bindgen_prelude::*;
use napi::Env;
use napi_derive::napi;

/// A QuickJS Atom representation in NAPI.
#[napi]
pub struct Atom {
  value: String,
}

#[napi]
impl Atom {
  #[napi(constructor)]
  pub fn new(value: String) -> Self {
    Atom { value }
  }

  /// Create an atom from a string.
  #[napi(factory)]
  pub fn from_str(name: String) -> Self {
    Atom { value: name }
  }

  /// Create an atom from a u32.
  #[napi(factory)]
  pub fn from_u32(val: u32) -> Self {
    Atom {
      value: val.to_string(),
    }
  }

  /// Create an atom from an i32.
  #[napi(factory)]
  pub fn from_i32(val: i32) -> Self {
    Atom {
      value: val.to_string(),
    }
  }

  /// Create an atom from a bool.
  #[napi(factory)]
  pub fn from_bool(val: bool) -> Self {
    Atom {
      value: val.to_string(),
    }
  }

  /// Create an atom from a f64.
  #[napi(factory)]
  pub fn from_f64(val: f64) -> Self {
    Atom {
      value: val.to_string(),
    }
  }

  /// Convert the atom to string.
  #[napi]
  pub fn to_string(&self) -> String {
    self.value.clone()
  }

  /// Convert the atom to a JS string.
  #[napi]
  pub fn to_js_string<'a>(&self, env: &'a Env) -> Result<Unknown<'a>> {
    env.create_string(&self.value)?.into_unknown(env)
  }

  /// Convert the atom to a JS value.
  #[napi]
  pub fn to_value<'a>(&self, env: &'a Env) -> Result<Unknown<'a>> {
    env.create_string(&self.value)?.into_unknown(env)
  }
}
