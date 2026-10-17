//! NAPI bindings for QuickJS `Atom`.

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Native primitive type identifiers exported for QuickJS Atom classification.
#[napi]
pub const U32: &str = "u32";
#[napi]
pub const I32: &str = "i32";
#[napi]
pub const U64: &str = "u64";
#[napi]
pub const I64: &str = "i64";
#[napi]
pub const F64: &str = "f64";
#[napi]
pub const F32: &str = "f32";
#[napi]
pub const BOOL: &str = "bool";
#[napi]
pub const STRING: &str = "string";

/// QuickJS Atom wrapper.
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

  /// Create an atom from a JavaScript value.
  #[napi(factory)]
  pub fn from_value(_val: Unknown) -> Result<Self> {
    Ok(Atom {
      value: "[value]".to_string(),
    })
  }

  /// Create an atom from a `u32`.
  #[napi(factory)]
  pub fn from_u32(val: u32) -> Result<Self> {
    Ok(Atom {
      value: val.to_string(),
    })
  }

  /// Create an atom from an `i32`.
  #[napi(factory)]
  pub fn from_i32(val: i32) -> Result<Self> {
    Ok(Atom {
      value: val.to_string(),
    })
  }

  /// Create an atom from a `bool`.
  #[napi(factory)]
  pub fn from_bool(val: bool) -> Result<Self> {
    Ok(Atom {
      value: val.to_string(),
    })
  }

  /// Create an atom from an `f64`.
  #[napi(factory)]
  pub fn from_f64(val: f64) -> Result<Self> {
    Ok(Atom {
      value: val.to_string(),
    })
  }

  /// Create an atom from a string.
  #[napi(factory)]
  pub fn from_str(name: String) -> Result<Self> {
    Ok(Atom { value: name })
  }

  /// Create an atom from a predefined atom string.
  #[napi(factory)]
  pub fn from_predefined(predefined: String) -> Result<Self> {
    Ok(Atom { value: predefined })
  }

  /// Convert the atom to a string.
  #[napi]
  pub fn to_string(&self) -> String {
    self.value.clone()
  }

  /// Convert the atom to a JS string.
  #[napi]
  pub fn to_js_string(&self) -> String {
    self.value.clone()
  }

  /// Convert the atom to a value representation.
  #[napi]
  pub fn to_value(&self) -> String {
    self.value.clone()
  }
}
