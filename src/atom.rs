use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::Value as JsonValue;

/// NAPI class wrapping a QuickJS Atom representation.
#[napi(js_name = "Atom")]
pub struct JsAtom {
  value: String,
}

#[napi(module_exports)]
pub fn init_atom_exports(exports: Object) -> Result<()> {
  if let Ok(mut atom_ctor) = exports.get_named_property::<Object>("Atom") {
    let in_js = serde_json::json!({
      "Undefined": "undefined",
      "Null": "null",
      "False": "false",
      "True": "true",
      "Length": "length",
      "Prototype": "prototype",
      "Constructor": "constructor",
      "Message": "message",
      "Name": "name",
      "Stack": "stack",
      "Symbol": "Symbol",
      "Object": "Object",
      "Array": "Array",
      "Error": "Error",
      "Function": "Function",
      "Promise": "Promise"
    });
    let in_rs = serde_json::json!({
      "U32": "u32",
      "I32": "i32",
      "U64": "u64",
      "I64": "i64",
      "F64": "f64",
      "F32": "f32",
      "BOOL": "bool",
      "STRING": "string"
    });
    let _ = atom_ctor.set_named_property("inJs", in_js);
    let _ = atom_ctor.set_named_property("inRs", in_rs);
  }
  Ok(())
}

#[napi]
impl JsAtom {
  #[napi(constructor)]
  pub fn new(val: String) -> Self {
    JsAtom { value: val }
  }

  /// Create an atom from a u32 value.
  #[napi(factory, js_name = "fromU32")]
  pub fn from_u32_camel(val: u32) -> Self {
    Self::from_u32(val)
  }

  #[napi(factory, js_name = "from_u32")]
  pub fn from_u32(val: u32) -> Self {
    JsAtom {
      value: val.to_string(),
    }
  }

  /// Create an atom from an i32 value.
  #[napi(factory, js_name = "fromI32")]
  pub fn from_i32_camel(val: i32) -> Self {
    Self::from_i32(val)
  }

  #[napi(factory, js_name = "from_i32")]
  pub fn from_i32(val: i32) -> Self {
    JsAtom {
      value: val.to_string(),
    }
  }

  /// Create an atom from a bool value.
  #[napi(factory, js_name = "fromBool")]
  pub fn from_bool_camel(val: bool) -> Self {
    Self::from_bool(val)
  }

  #[napi(factory, js_name = "from_bool")]
  pub fn from_bool(val: bool) -> Self {
    JsAtom {
      value: val.to_string(),
    }
  }

  /// Create an atom from an f64 value.
  #[napi(factory, js_name = "fromF64")]
  pub fn from_f64_camel(val: f64) -> Self {
    Self::from_f64(val)
  }

  #[napi(factory, js_name = "from_f64")]
  pub fn from_f64(val: f64) -> Self {
    JsAtom {
      value: val.to_string(),
    }
  }

  /// Create an atom from a string.
  #[napi(factory, js_name = "fromStr")]
  pub fn from_str_camel(name: String) -> Self {
    Self::from_str(name)
  }

  #[napi(factory, js_name = "from_str")]
  pub fn from_str(name: String) -> Self {
    JsAtom { value: name }
  }

  /// Create an atom from a predefined string name.
  #[napi(factory, js_name = "fromPredefined")]
  pub fn from_predefined_camel(name: String) -> Self {
    Self::from_predefined(name)
  }

  #[napi(factory, js_name = "from_predefined")]
  pub fn from_predefined(name: String) -> Self {
    JsAtom { value: name }
  }

  /// Create an atom from a JS/JSON value representation.
  #[napi(factory, js_name = "fromValue")]
  pub fn from_value_camel(val: JsonValue) -> Self {
    Self::from_value(val)
  }

  #[napi(factory, js_name = "from_value")]
  pub fn from_value(val: JsonValue) -> Self {
    let s = match val {
      JsonValue::String(s) => s,
      JsonValue::Number(n) => n.to_string(),
      JsonValue::Bool(b) => b.to_string(),
      other => serde_json::to_string(&other).unwrap_or_default(),
    };
    JsAtom { value: s }
  }

  /// Convert the atom to string.
  #[napi(js_name = "toString")]
  pub fn to_string_js(&self) -> String {
    self.value.clone()
  }

  #[napi(js_name = "to_string")]
  pub fn to_string(&self) -> String {
    self.value.clone()
  }

  /// Convert the atom to a JavaScript string representation.
  #[napi(js_name = "toJsString")]
  pub fn to_js_string_camel(&self) -> String {
    self.value.clone()
  }

  #[napi(js_name = "to_js_string")]
  pub fn to_js_string(&self) -> String {
    self.value.clone()
  }

  /// Convert the atom to a JS value.
  #[napi(js_name = "toValue")]
  pub fn to_value_camel(&self) -> JsonValue {
    self.to_value()
  }

  #[napi(js_name = "to_value")]
  pub fn to_value(&self) -> JsonValue {
    JsonValue::String(self.value.clone())
  }

  /// Instance getter for inJs
  #[napi(getter, js_name = "inJs")]
  pub fn in_js(&self) -> JsonValue {
    serde_json::json!({
      "Undefined": "undefined",
      "Null": "null",
      "False": "false",
      "True": "true",
      "Length": "length",
      "Prototype": "prototype",
      "Constructor": "constructor",
      "Message": "message",
      "Name": "name",
      "Stack": "stack",
      "Symbol": "Symbol",
      "Object": "Object",
      "Array": "Array",
      "Error": "Error",
      "Function": "Function",
      "Promise": "Promise"
    })
  }

  /// Instance getter for inRs
  #[napi(getter, js_name = "inRs")]
  pub fn in_rs(&self) -> JsonValue {
    serde_json::json!({
      "U32": "u32",
      "I32": "i32",
      "U64": "u64",
      "I64": "i64",
      "F64": "f64",
      "F32": "f32",
      "BOOL": "bool",
      "STRING": "string"
    })
  }
}
