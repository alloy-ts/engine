use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::{atom::PredefinedAtom, Atom as QjsAtom};
use serde_json::Value as JsonValue;

use crate::context::JsContext;

fn u32_to_predefined_atom(id: u32) -> Option<PredefinedAtom> {
  if (PredefinedAtom::Null as u32..=PredefinedAtom::SymbolUnscopables as u32).contains(&id) {
    // Safety: id is verified to be within the contiguous range of QuickJS predefined atoms.
    Some(unsafe { std::mem::transmute(id) })
  } else {
    None
  }
}

#[napi(js_name = "Atom")]
pub struct Atom {
  pub value: String,
}

#[napi]
impl Atom {
  /// Create an Atom from a JS Value in a given Context.
  #[napi(factory, js_name = "from_value")]
  pub fn from_value(ctx: &JsContext, val: JsonValue) -> Result<Self> {
    match val {
      JsonValue::Null => Self::from_str(ctx, "null".to_string()),
      JsonValue::Bool(b) => Self::from_bool(ctx, b),
      JsonValue::Number(ref n) => {
        if let Some(u) = n.as_u64() {
          if u <= u32::MAX as u64 {
            return Self::from_u32(ctx, u as u32);
          }
        }
        if let Some(i) = n.as_i64() {
          if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
            return Self::from_i32(ctx, i as i32);
          }
        }
        if let Some(f) = n.as_f64() {
          return Self::from_f64(ctx, f);
        }
        Self::from_str(ctx, n.to_string())
      }
      JsonValue::String(s) => Self::from_str(ctx, s),
      JsonValue::Array(_) | JsonValue::Object(_) => {
        let json_str = serde_json::to_string(&val).map_err(|e| Error::from_reason(e.to_string()))?;
        ctx.inner.with(|c| {
          let qjs_val = c
            .json_parse(json_str)
            .map_err(|e| Error::from_reason(e.to_string()))?;
          let atom = QjsAtom::from_value(c.clone(), &qjs_val)
            .map_err(|e| Error::from_reason(e.to_string()))?;
          let val_str = atom
            .to_string()
            .map_err(|e| Error::from_reason(e.to_string()))?;
          Ok(Self { value: val_str })
        })
      }
    }
  }

  /// Alias for `from_value` in camelCase for JS convenience.
  #[napi(factory, js_name = "fromValue")]
  pub fn from_value_camel(ctx: &JsContext, val: JsonValue) -> Result<Self> {
    Self::from_value(ctx, val)
  }

  /// Create an Atom from a u32 integer in a given Context.
  #[napi(factory, js_name = "from_u32")]
  pub fn from_u32(ctx: &JsContext, val: u32) -> Result<Self> {
    ctx.inner.with(|c| {
      let atom = QjsAtom::from_u32(c.clone(), val)
        .map_err(|e| Error::from_reason(e.to_string()))?;
      let val_str = atom
        .to_string()
        .map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { value: val_str })
    })
  }

  /// Alias for `from_u32` in camelCase for JS convenience.
  #[napi(factory, js_name = "fromU32")]
  pub fn from_u32_camel(ctx: &JsContext, val: u32) -> Result<Self> {
    Self::from_u32(ctx, val)
  }

  /// Create an Atom from an i32 integer in a given Context.
  #[napi(factory, js_name = "from_i32")]
  pub fn from_i32(ctx: &JsContext, val: i32) -> Result<Self> {
    ctx.inner.with(|c| {
      let atom = QjsAtom::from_i32(c.clone(), val)
        .map_err(|e| Error::from_reason(e.to_string()))?;
      let val_str = atom
        .to_string()
        .map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { value: val_str })
    })
  }

  /// Alias for `from_i32` in camelCase for JS convenience.
  #[napi(factory, js_name = "fromI32")]
  pub fn from_i32_camel(ctx: &JsContext, val: i32) -> Result<Self> {
    Self::from_i32(ctx, val)
  }

  /// Create an Atom from a bool in a given Context.
  #[napi(factory, js_name = "from_bool")]
  pub fn from_bool(ctx: &JsContext, val: bool) -> Result<Self> {
    ctx.inner.with(|c| {
      let atom = QjsAtom::from_bool(c.clone(), val)
        .map_err(|e| Error::from_reason(e.to_string()))?;
      let val_str = atom
        .to_string()
        .map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { value: val_str })
    })
  }

  /// Alias for `from_bool` in camelCase for JS convenience.
  #[napi(factory, js_name = "fromBool")]
  pub fn from_bool_camel(ctx: &JsContext, val: bool) -> Result<Self> {
    Self::from_bool(ctx, val)
  }

  /// Create an Atom from a f64 in a given Context.
  #[napi(factory, js_name = "from_f64")]
  pub fn from_f64(ctx: &JsContext, val: f64) -> Result<Self> {
    ctx.inner.with(|c| {
      let atom = QjsAtom::from_f64(c.clone(), val)
        .map_err(|e| Error::from_reason(e.to_string()))?;
      let val_str = atom
        .to_string()
        .map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { value: val_str })
    })
  }

  /// Alias for `from_f64` in camelCase for JS convenience.
  #[napi(factory, js_name = "fromF64")]
  pub fn from_f64_camel(ctx: &JsContext, val: f64) -> Result<Self> {
    Self::from_f64(ctx, val)
  }

  /// Create an Atom from a string in a given Context.
  #[napi(factory, js_name = "from_str")]
  pub fn from_str(ctx: &JsContext, name: String) -> Result<Self> {
    ctx.inner.with(|c| {
      let atom = QjsAtom::from_str(c.clone(), &name)
        .map_err(|e| Error::from_reason(e.to_string()))?;
      let val_str = atom
        .to_string()
        .map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { value: val_str })
    })
  }

  /// Alias for `from_str` in camelCase for JS convenience.
  #[napi(factory, js_name = "fromStr")]
  pub fn from_str_camel(ctx: &JsContext, name: String) -> Result<Self> {
    Self::from_str(ctx, name)
  }

  /// Create an Atom from a predefined atom in a given Context.
  #[napi(factory, js_name = "from_predefined")]
  pub fn from_predefined(ctx: &JsContext, predefined: Either<u32, String>) -> Result<Self> {
    ctx.inner.with(|c| {
      let atom = match predefined {
        Either::A(id) => {
          let predef = u32_to_predefined_atom(id)
            .ok_or_else(|| Error::from_reason(format!("Invalid predefined atom ID: {id}")))?;
          QjsAtom::from_predefined(c.clone(), predef)
        }
        Either::B(name) => QjsAtom::from_str(c.clone(), &name)
          .map_err(|e| Error::from_reason(e.to_string()))?,
      };
      let val_str = atom
        .to_string()
        .map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { value: val_str })
    })
  }

  /// Alias for `from_predefined` in camelCase for JS convenience.
  #[napi(factory, js_name = "fromPredefined")]
  pub fn from_predefined_camel(ctx: &JsContext, predefined: Either<u32, String>) -> Result<Self> {
    Self::from_predefined(ctx, predefined)
  }

  /// Convert the Atom to a JavaScript string.
  #[napi(js_name = "to_string")]
  pub fn to_string(&self) -> String {
    self.value.clone()
  }

  /// Alias for `to_string` standard toString in JS.
  #[napi(js_name = "toString")]
  pub fn to_string_camel(&self) -> String {
    self.value.clone()
  }

  /// Convert the Atom to a JavaScript string.
  #[napi(js_name = "to_js_string")]
  pub fn to_js_string(&self) -> String {
    self.value.clone()
  }

  /// Alias for `to_js_string` in camelCase for JS convenience.
  #[napi(js_name = "toJsString")]
  pub fn to_js_string_camel(&self) -> String {
    self.value.clone()
  }

  /// Convert the Atom to a JavaScript value.
  #[napi(js_name = "to_value")]
  pub fn to_value(&self) -> JsonValue {
    JsonValue::String(self.value.clone())
  }

  /// Alias for `to_value` in camelCase for JS convenience.
  #[napi(js_name = "toValue")]
  pub fn to_value_camel(&self) -> JsonValue {
    JsonValue::String(self.value.clone())
  }
}
