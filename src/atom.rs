use napi_derive::napi;

/// A QuickJS Atom wrapper exposed via NAPI.
///
/// In QuickJS atoms represent interned strings, symbols, or numeric property indices.
#[napi(js_name = "Atom")]
pub struct Atom {
  value: String,
}

#[napi]
impl Atom {
  /// Create a new Atom with a string representation.
  #[napi(constructor)]
  pub fn new(value: String) -> Self {
    Atom { value }
  }

  /// Create an atom from a string.
  #[napi(js_name = "fromStr")]
  pub fn from_str(name: String) -> Self {
    Atom { value: name }
  }

  /// Create an atom from a `u32`.
  #[napi(js_name = "fromU32")]
  pub fn from_u32(val: u32) -> Self {
    Atom {
      value: val.to_string(),
    }
  }

  /// Create an atom from an `i32`.
  #[napi(js_name = "fromI32")]
  pub fn from_i32(val: i32) -> Self {
    Atom {
      value: val.to_string(),
    }
  }

  /// Create an atom from a `bool`.
  #[napi(js_name = "fromBool")]
  pub fn from_bool(val: bool) -> Self {
    Atom {
      value: val.to_string(),
    }
  }

  /// Create an atom from a `f64`.
  #[napi(js_name = "fromF64")]
  pub fn from_f64(val: f64) -> Self {
    Atom {
      value: val.to_string(),
    }
  }

  /// Convert the atom to a JavaScript string.
  #[napi(js_name = "toString")]
  pub fn to_string(&self) -> String {
    self.value.clone()
  }

  /// Convert the atom to a JavaScript value.
  #[napi(js_name = "toValue")]
  pub fn to_value(&self) -> String {
    self.value.clone()
  }

  /// Getter for the atom string value.
  #[napi(getter)]
  pub fn value(&self) -> String {
    self.value.clone()
  }
}
