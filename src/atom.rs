use napi_derive::napi;

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

    /// Create an Atom from a Rust string.
    #[napi(factory, js_name = "from_str")]
    pub fn from_str(value: String) -> napi::Result<Self> {
        Ok(Self { value })
    }

    /// Create an Atom from a u32 index.
    #[napi(factory, js_name = "from_u32")]
    pub fn from_u32(val: u32) -> napi::Result<Self> {
        Ok(Self {
            value: val.to_string(),
        })
    }

    /// Create an Atom from an i32 index.
    #[napi(factory, js_name = "from_i32")]
    pub fn from_i32(val: i32) -> napi::Result<Self> {
        Ok(Self {
            value: val.to_string(),
        })
    }

    /// Create an Atom from a bool.
    #[napi(factory, js_name = "from_bool")]
    pub fn from_bool(val: bool) -> napi::Result<Self> {
        Ok(Self {
            value: val.to_string(),
        })
    }

    /// Create an Atom from an f64.
    #[napi(factory, js_name = "from_f64")]
    pub fn from_f64(val: f64) -> napi::Result<Self> {
        Ok(Self {
            value: val.to_string(),
        })
    }

    /// Convert the atom to its string representation.
    #[napi(js_name = "to_string")]
    pub fn to_string_js(&self) -> String {
        self.value.clone()
    }

    /// Convert the atom to its string value.
    #[napi(js_name = "to_value")]
    pub fn to_value(&self) -> String {
        self.value.clone()
    }
}
