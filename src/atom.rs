use napi_derive::napi;
use serde_json::Value as JsonValue;

/// NAPI wrapper around QuickJS Atom representation.
#[napi(js_name = "Atom")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsAtom {
    pub(crate) value: String,
}

impl Default for JsAtom {
    fn default() -> Self {
        Self {
            value: String::new(),
        }
    }
}

#[napi]
impl JsAtom {
    /// Create a new Atom from a string.
    #[napi(constructor)]
    pub fn new(value: String) -> napi::Result<Self> {
        Ok(Self { value })
    }

    /// Create an Atom from a JS value.
    #[napi(factory, js_name = "from_value")]
    pub fn from_value(val: JsonValue) -> napi::Result<Self> {
        let s = match val {
            JsonValue::String(s) => s,
            JsonValue::Number(n) => n.to_string(),
            JsonValue::Bool(b) => b.to_string(),
            JsonValue::Null => "null".to_string(),
            other => serde_json::to_string(&other).unwrap_or_default(),
        };
        Ok(Self { value: s })
    }

    /// Create an Atom from a JS value (camelCase alias).
    #[napi(factory, js_name = "fromValue")]
    pub fn from_value_camel(val: JsonValue) -> napi::Result<Self> {
        Self::from_value(val)
    }

    /// Create an Atom from a u32 index.
    #[napi(factory, js_name = "from_u32")]
    pub fn from_u32(val: u32) -> napi::Result<Self> {
        Ok(Self {
            value: val.to_string(),
        })
    }

    /// Create an Atom from a u32 index (camelCase alias).
    #[napi(factory, js_name = "fromU32")]
    pub fn from_u32_camel(val: u32) -> napi::Result<Self> {
        Self::from_u32(val)
    }

    /// Create an Atom from an i32 index.
    #[napi(factory, js_name = "from_i32")]
    pub fn from_i32(val: i32) -> napi::Result<Self> {
        Ok(Self {
            value: val.to_string(),
        })
    }

    /// Create an Atom from an i32 index (camelCase alias).
    #[napi(factory, js_name = "fromI32")]
    pub fn from_i32_camel(val: i32) -> napi::Result<Self> {
        Self::from_i32(val)
    }

    /// Create an Atom from a bool.
    #[napi(factory, js_name = "from_bool")]
    pub fn from_bool(val: bool) -> napi::Result<Self> {
        Ok(Self {
            value: val.to_string(),
        })
    }

    /// Create an Atom from a bool (camelCase alias).
    #[napi(factory, js_name = "fromBool")]
    pub fn from_bool_camel(val: bool) -> napi::Result<Self> {
        Self::from_bool(val)
    }

    /// Create an Atom from an f64.
    #[napi(factory, js_name = "from_f64")]
    pub fn from_f64(val: f64) -> napi::Result<Self> {
        Ok(Self {
            value: val.to_string(),
        })
    }

    /// Create an Atom from an f64 (camelCase alias).
    #[napi(factory, js_name = "fromF64")]
    pub fn from_f64_camel(val: f64) -> napi::Result<Self> {
        Self::from_f64(val)
    }

    /// Create an Atom from a string.
    #[napi(factory, js_name = "from_str")]
    pub fn from_str(value: String) -> napi::Result<Self> {
        Ok(Self { value })
    }

    /// Create an Atom from a string (camelCase alias).
    #[napi(factory, js_name = "fromStr")]
    pub fn from_str_camel(value: String) -> napi::Result<Self> {
        Self::from_str(value)
    }

    /// Create an Atom from a predefined atom name.
    #[napi(factory, js_name = "from_predefined")]
    pub fn from_predefined(predefined: String) -> napi::Result<Self> {
        Ok(Self { value: predefined })
    }

    /// Create an Atom from a predefined atom name (camelCase alias).
    #[napi(factory, js_name = "fromPredefined")]
    pub fn from_predefined_camel(predefined: String) -> napi::Result<Self> {
        Self::from_predefined(predefined)
    }

    /// Convert the atom to its string representation.
    #[napi(js_name = "to_string")]
    pub fn to_string_js(&self) -> String {
        self.value.clone()
    }

    /// Convert the atom to its string representation (camelCase alias).
    #[napi(js_name = "toString")]
    pub fn to_string_camel(&self) -> String {
        self.value.clone()
    }

    /// Convert the atom to its string value.
    #[napi(js_name = "to_value")]
    pub fn to_value(&self) -> String {
        self.value.clone()
    }

    /// Convert the atom to its string value (camelCase alias).
    #[napi(js_name = "toValue")]
    pub fn to_value_camel(&self) -> String {
        self.value.clone()
    }
}
