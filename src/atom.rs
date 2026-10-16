//! NAPI bindings for QuickJS [`Atom`].

use napi_derive::napi;

/// NAPI wrapper around a QuickJS Atom (interned string / symbol / property key).
#[napi]
pub struct Atom {
  value: String,
}

#[napi]
impl Atom {
  /// Create an Atom from a string.
  #[napi(constructor)]
  pub fn new(name: String) -> Self {
    Atom { value: name }
  }

  /// Create an Atom from a Rust/JS string.
  #[napi(js_name = "from_str")]
  pub fn from_str(name: String) -> Self {
    Atom { value: name }
  }

  /// Create an Atom from a u32 number.
  #[napi(js_name = "from_u32")]
  pub fn from_u32(val: u32) -> Self {
    Atom { value: val.to_string() }
  }

  /// Create an Atom from an i32 number.
  #[napi(js_name = "from_i32")]
  pub fn from_i32(val: i32) -> Self {
    Atom { value: val.to_string() }
  }

  /// Create an Atom from an f64 number.
  #[napi(js_name = "from_f64")]
  pub fn from_f64(val: f64) -> Self {
    Atom { value: val.to_string() }
  }

  /// Create an Atom from a boolean.
  #[napi(js_name = "from_bool")]
  pub fn from_bool(val: bool) -> Self {
    Atom { value: val.to_string() }
  }

  /// Convert the Atom back to a string.
  #[napi(js_name = "to_string")]
  pub fn to_string(&self) -> String {
    self.value.clone()
  }
}
