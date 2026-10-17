use napi::bindgen_prelude::Result as NapiResult;
use napi::Error;
use napi_derive::napi;
use rquickjs::{Atom as QjsAtom, Context as QjsContext, Runtime as QjsRuntime, Value};
use std::collections::HashMap;

fn to_napi_err(e: rquickjs::Error) -> Error {
    Error::from_reason(e.to_string())
}

/// In QuickJS atoms are similar to interned strings but with additional uses.
/// A symbol for instance is just an atom.
#[napi(js_name = "Atom")]
pub struct Atom {
    ctx: QjsContext,
    value: String,
}

#[napi]
impl Atom {
    #[napi(constructor)]
    pub fn new(value: Option<String>) -> Self {
        let runtime = QjsRuntime::new().expect("failed to create QuickJS runtime");
        let ctx = QjsContext::full(&runtime).expect("failed to create QuickJS context");
        let str_val = value.unwrap_or_default();
        ctx.with(|c| {
            let _atom = QjsAtom::from_str(c.clone(), &str_val);
        });
        Atom {
            ctx,
            value: str_val,
        }
    }

    /// Create an atom from a predefined atom name.
    #[napi(factory)]
    pub fn from_predefined(predefined: String) -> Self {
        let runtime = QjsRuntime::new().expect("failed to create QuickJS runtime");
        let ctx = QjsContext::full(&runtime).expect("failed to create QuickJS context");
        let str_val = ctx.with(|c| {
            let atom = QjsAtom::from_str(c.clone(), &predefined).expect("failed atom creation");
            atom.to_string().expect("failed to stringify atom")
        });
        Atom {
            ctx,
            value: str_val,
        }
    }

    #[napi(factory, js_name = "from_predefined")]
    pub fn from_predefined_snake(predefined: String) -> Self {
        Self::from_predefined(predefined)
    }

    /// Create an atom from a `u32`.
    #[napi(factory)]
    pub fn from_u32(val: u32) -> Self {
        let runtime = QjsRuntime::new().expect("failed to create QuickJS runtime");
        let ctx = QjsContext::full(&runtime).expect("failed to create QuickJS context");
        let str_val = ctx.with(|c| {
            let atom = QjsAtom::from_u32(c.clone(), val).expect("failed atom creation");
            atom.to_string().expect("failed to stringify atom")
        });
        Atom {
            ctx,
            value: str_val,
        }
    }

    #[napi(factory, js_name = "from_u32")]
    pub fn from_u32_snake(val: u32) -> Self {
        Self::from_u32(val)
    }

    /// Create an atom from an `i32`.
    #[napi(factory)]
    pub fn from_i32(val: i32) -> Self {
        let runtime = QjsRuntime::new().expect("failed to create QuickJS runtime");
        let ctx = QjsContext::full(&runtime).expect("failed to create QuickJS context");
        let str_val = ctx.with(|c| {
            let atom = QjsAtom::from_i32(c.clone(), val).expect("failed atom creation");
            atom.to_string().expect("failed to stringify atom")
        });
        Atom {
            ctx,
            value: str_val,
        }
    }

    #[napi(factory, js_name = "from_i32")]
    pub fn from_i32_snake(val: i32) -> Self {
        Self::from_i32(val)
    }

    /// Create an atom from a `bool`.
    #[napi(factory)]
    pub fn from_bool(val: bool) -> Self {
        let runtime = QjsRuntime::new().expect("failed to create QuickJS runtime");
        let ctx = QjsContext::full(&runtime).expect("failed to create QuickJS context");
        let str_val = ctx.with(|c| {
            let atom = QjsAtom::from_bool(c.clone(), val).expect("failed atom creation");
            atom.to_string().expect("failed to stringify atom")
        });
        Atom {
            ctx,
            value: str_val,
        }
    }

    #[napi(factory, js_name = "from_bool")]
    pub fn from_bool_snake(val: bool) -> Self {
        Self::from_bool(val)
    }

    /// Create an atom from a `f64`.
    #[napi(factory)]
    pub fn from_f64(val: f64) -> Self {
        let runtime = QjsRuntime::new().expect("failed to create QuickJS runtime");
        let ctx = QjsContext::full(&runtime).expect("failed to create QuickJS context");
        let str_val = ctx.with(|c| {
            let atom = QjsAtom::from_f64(c.clone(), val).expect("failed atom creation");
            atom.to_string().expect("failed to stringify atom")
        });
        Atom {
            ctx,
            value: str_val,
        }
    }

    #[napi(factory, js_name = "from_f64")]
    pub fn from_f64_snake(val: f64) -> Self {
        Self::from_f64(val)
    }

    /// Create an atom from a native str.
    #[napi(factory)]
    pub fn from_str(name: String) -> Self {
        let runtime = QjsRuntime::new().expect("failed to create QuickJS runtime");
        let ctx = QjsContext::full(&runtime).expect("failed to create QuickJS context");
        let str_val = ctx.with(|c| {
            let atom = QjsAtom::from_str(c.clone(), &name).expect("failed atom creation");
            atom.to_string().expect("failed to stringify atom")
        });
        Atom {
            ctx,
            value: str_val,
        }
    }

    #[napi(factory, js_name = "from_str")]
    pub fn from_str_snake(name: String) -> Self {
        Self::from_str(name)
    }

    /// Create an atom from a JavaScript value representation (stringified or primitive string).
    #[napi(factory)]
    pub fn from_value(val: String) -> Self {
        let runtime = QjsRuntime::new().expect("failed to create QuickJS runtime");
        let ctx = QjsContext::full(&runtime).expect("failed to create QuickJS context");
        let str_val = ctx.with(|c| {
            let js_val: Value = c.eval(val.as_str()).unwrap_or_else(|_| rquickjs::String::from_str(c.clone(), &val).unwrap().into_value());
            let atom = QjsAtom::from_value(c.clone(), &js_val).expect("failed atom creation");
            atom.to_string().expect("failed to stringify atom")
        });
        Atom {
            ctx,
            value: str_val,
        }
    }

    #[napi(factory, js_name = "from_value")]
    pub fn from_value_snake(val: String) -> Self {
        Self::from_value(val)
    }

    /// Convert the atom to a JavaScript string.
    #[napi]
    pub fn to_string(&self) -> NapiResult<String> {
        self.ctx.with(|c| {
            let atom = QjsAtom::from_str(c.clone(), &self.value).map_err(to_napi_err)?;
            atom.to_string().map_err(to_napi_err)
        })
    }

    #[napi(js_name = "to_string")]
    pub fn to_string_snake(&self) -> NapiResult<String> {
        self.to_string()
    }

    /// Convert the atom to a JavaScript string representation.
    #[napi]
    pub fn to_js_string(&self) -> NapiResult<String> {
        self.ctx.with(|c| {
            let atom = QjsAtom::from_str(c.clone(), &self.value).map_err(to_napi_err)?;
            let js_str = atom.to_js_string().map_err(to_napi_err)?;
            js_str.to_string().map_err(to_napi_err)
        })
    }

    #[napi(js_name = "to_js_string")]
    pub fn to_js_string_snake(&self) -> NapiResult<String> {
        self.to_js_string()
    }

    /// Convert the atom to a JavaScript value.
    #[napi]
    pub fn to_value(&self) -> NapiResult<String> {
        Ok(self.value.clone())
    }

    #[napi(js_name = "to_value")]
    pub fn to_value_snake(&self) -> NapiResult<String> {
        self.to_value()
    }

    #[napi(getter, js_name = "inJs")]
    pub fn in_js(&self) -> HashMap<String, String> {
        Self::get_in_js()
    }

    #[napi(getter, js_name = "inRs")]
    pub fn in_rs(&self) -> HashMap<String, String> {
        Self::get_in_rs()
    }

    pub fn get_in_js() -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("Undefined".to_string(), "undefined".to_string());
        map.insert("Null".to_string(), "null".to_string());
        map.insert("False".to_string(), "false".to_string());
        map.insert("True".to_string(), "true".to_string());
        map.insert("Object".to_string(), "Object".to_string());
        map.insert("Array".to_string(), "Array".to_string());
        map.insert("String".to_string(), "String".to_string());
        map.insert("Number".to_string(), "Number".to_string());
        map.insert("Boolean".to_string(), "Boolean".to_string());
        map.insert("Symbol".to_string(), "Symbol".to_string());
        map.insert("Function".to_string(), "Function".to_string());
        map.insert("Error".to_string(), "Error".to_string());
        map.insert("Promise".to_string(), "Promise".to_string());
        map.insert("Length".to_string(), "length".to_string());
        map.insert("Prototype".to_string(), "prototype".to_string());
        map.insert("Constructor".to_string(), "constructor".to_string());
        map.insert("Value".to_string(), "value".to_string());
        map.insert("Name".to_string(), "name".to_string());
        map.insert("Message".to_string(), "message".to_string());
        map.insert("Stack".to_string(), "stack".to_string());
        map.insert("ToString".to_string(), "toString".to_string());
        map.insert("ValueOf".to_string(), "valueOf".to_string());
        map.insert("Eval".to_string(), "eval".to_string());
        map.insert("GlobalThis".to_string(), "globalThis".to_string());
        map
    }

    pub fn get_in_rs() -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("U32".to_string(), "u32".to_string());
        map.insert("I32".to_string(), "i32".to_string());
        map.insert("U64".to_string(), "u64".to_string());
        map.insert("I64".to_string(), "i64".to_string());
        map.insert("F64".to_string(), "f64".to_string());
        map.insert("F32".to_string(), "f32".to_string());
        map.insert("BOOL".to_string(), "bool".to_string());
        map.insert("STRING".to_string(), "string".to_string());
        map
    }
}
