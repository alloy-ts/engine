//! QuickJS Atom NAPI bindings and Rust primitives.

use napi::bindgen_prelude::{ToNapiValue, Unknown};
use napi::{Env, Result as NapiResult};
use napi_derive::napi;

pub use rquickjs::Atom;

/// NAPI-exposed wrapper around QuickJS `Atom`.
#[napi(js_name = "Atom")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JsAtom {
  pub(crate) name: String,
}

#[napi]
impl JsAtom {
  /// Create an atom from a string.
  #[napi(constructor)]
  pub fn new(_env: Env, value: String) -> Self {
    JsAtom { name: value }
  }

  /// Create an atom from a Rust/JS string.
  #[napi(factory, js_name = "fromStr")]
  pub fn from_str(value: String) -> Self {
    JsAtom { name: value }
  }

  /// Create an atom from a u32 number.
  #[napi(factory, js_name = "fromU32")]
  pub fn from_u32(value: u32) -> Self {
    JsAtom {
      name: value.to_string(),
    }
  }

  /// Create an atom from an i32 number.
  #[napi(factory, js_name = "fromI32")]
  pub fn from_i32(value: i32) -> Self {
    JsAtom {
      name: value.to_string(),
    }
  }

  /// Create an atom from a boolean value.
  #[napi(factory, js_name = "fromBool")]
  pub fn from_bool(value: bool) -> Self {
    JsAtom {
      name: value.to_string(),
    }
  }

  /// Create an atom from an f64 number.
  #[napi(factory, js_name = "fromF64")]
  pub fn from_f64(value: f64) -> Self {
    JsAtom {
      name: value.to_string(),
    }
  }

  /// Convert atom to a string representation.
  #[napi(js_name = "toString")]
  pub fn to_string_js(&self) -> String {
    self.name.clone()
  }

  /// Convert atom to JS value.
  #[napi(js_name = "toValue")]
  pub fn to_value<'env>(&self, env: &'env Env) -> NapiResult<Unknown<'env>> {
    let str_val = env.create_string(&self.name)?;
    str_val.into_unknown(env)
  }

  /// Check equality with another Atom.
  #[napi]
  pub fn equals(&self, other: &JsAtom) -> bool {
    self.name == other.name
  }
}

impl JsAtom {
  pub fn to_string_res(&self) -> Result<String, rquickjs::Error> {
    Ok(self.name.clone())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_atom_from_str() {
    let rt = rquickjs::Runtime::new().unwrap();
    let ctx = rquickjs::Context::full(&rt).unwrap();
    ctx.with(|ctx| {
      let atom = Atom::from_str(ctx, "hello").unwrap();
      let s: String = atom.to_string().unwrap();
      assert_eq!(s, "hello");
    });
  }

  #[test]
  fn test_atom_from_u32() {
    let rt = rquickjs::Runtime::new().unwrap();
    let ctx = rquickjs::Context::full(&rt).unwrap();
    ctx.with(|ctx| {
      let atom = Atom::from_u32(ctx, 42).unwrap();
      let s: String = atom.to_string().unwrap();
      assert_eq!(s, "42");
    });
  }

  #[test]
  fn test_atom_from_bool() {
    let rt = rquickjs::Runtime::new().unwrap();
    let ctx = rquickjs::Context::full(&rt).unwrap();
    ctx.with(|ctx| {
      let atom = Atom::from_bool(ctx, true).unwrap();
      let s: String = atom.to_string().unwrap();
      assert_eq!(s, "true");
    });
  }
}
