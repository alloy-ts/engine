use napi::bindgen_prelude::Result as NapiResult;
use napi::Error;
use napi_derive::napi;
use rquickjs::{Atom as QjsAtom, Context as QjsContext, Runtime as QjsRuntime, Value};

fn to_napi_err(e: rquickjs::Error) -> Error {
    Error::from_reason(e.to_string())
}

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

#[napi(object)]
pub struct AtomInJs {
    #[napi(js_name = "Undefined")]
    pub undefined: String,
    #[napi(js_name = "Null")]
    pub null: String,
    #[napi(js_name = "True")]
    pub true_val: String,
    #[napi(js_name = "False")]
    pub false_val: String,
    #[napi(js_name = "Length")]
    pub length: String,
    #[napi(js_name = "Name")]
    pub name: String,
    #[napi(js_name = "Object")]
    pub object: String,
    #[napi(js_name = "Array")]
    pub array: String,
    #[napi(js_name = "String")]
    pub string: String,
    #[napi(js_name = "Number")]
    pub number: String,
    #[napi(js_name = "Boolean")]
    pub boolean: String,
    #[napi(js_name = "Symbol")]
    pub symbol: String,
    #[napi(js_name = "Function")]
    pub function: String,
    #[napi(js_name = "Promise")]
    pub promise: String,
    #[napi(js_name = "Value")]
    pub value: String,
    #[napi(js_name = "ToString")]
    pub to_string: String,
    #[napi(js_name = "ValueOf")]
    pub value_of: String,
}

#[napi(object)]
pub struct AtomInRs {
    #[napi(js_name = "U32")]
    pub u32: String,
    #[napi(js_name = "I32")]
    pub i32: String,
    #[napi(js_name = "U64")]
    pub u64: String,
    #[napi(js_name = "I64")]
    pub i64: String,
    #[napi(js_name = "F64")]
    pub f64: String,
    #[napi(js_name = "F32")]
    pub f32: String,
    #[napi(js_name = "BOOL")]
    pub bool: String,
    #[napi(js_name = "STRING")]
    pub string: String,
}

/// In QuickJS atoms are similar to interned strings but with additional uses.
/// A symbol for instance is just an atom.
#[napi]
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

    #[napi(getter, js_name = "inJs")]
    pub fn in_js(&self) -> AtomInJs {
        AtomInJs {
            undefined: "undefined".to_string(),
            null: "null".to_string(),
            true_val: "true".to_string(),
            false_val: "false".to_string(),
            length: "length".to_string(),
            name: "name".to_string(),
            object: "Object".to_string(),
            array: "Array".to_string(),
            string: "String".to_string(),
            number: "Number".to_string(),
            boolean: "Boolean".to_string(),
            symbol: "Symbol".to_string(),
            function: "Function".to_string(),
            promise: "Promise".to_string(),
            value: "value".to_string(),
            to_string: "toString".to_string(),
            value_of: "valueOf".to_string(),
        }
    }

    #[napi(getter, js_name = "inRs")]
    pub fn in_rs(&self) -> AtomInRs {
        AtomInRs {
            u32: "u32".to_string(),
            i32: "i32".to_string(),
            u64: "u64".to_string(),
            i64: "i64".to_string(),
            f64: "f64".to_string(),
            f32: "f32".to_string(),
            bool: "bool".to_string(),
            string: "string".to_string(),
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

    /// Create an atom from a JavaScript value representation (stringified or primitive string).
    #[napi(factory)]
    pub fn from_value(val: String) -> Self {
        let runtime = QjsRuntime::new().expect("failed to create QuickJS runtime");
        let ctx = QjsContext::full(&runtime).expect("failed to create QuickJS context");
        let str_val = ctx.with(|c| {
            let js_val: Value = c.eval(val.as_str()).unwrap_or_else(|_| Value::new_undefined(c.clone()));
            let atom = QjsAtom::from_value(c.clone(), &js_val).expect("failed atom creation");
            atom.to_string().expect("failed to stringify atom")
        });
        Atom {
            ctx,
            value: str_val,
        }
    }

    /// Convert the atom to a JavaScript string.
    #[napi]
    pub fn to_string(&self) -> NapiResult<String> {
        self.ctx.with(|c| {
            let atom = QjsAtom::from_str(c.clone(), &self.value).map_err(to_napi_err)?;
            atom.to_string().map_err(to_napi_err)
        })
    }

    /// Convert the atom to a JavaScript string (snake_case alias).
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

    /// Convert the atom to a JavaScript string representation (snake_case alias).
    #[napi(js_name = "to_js_string")]
    pub fn to_js_string_snake(&self) -> NapiResult<String> {
        self.to_js_string()
    }

    /// Convert the atom to a JavaScript value.
    #[napi]
    pub fn to_value(&self) -> NapiResult<String> {
        Ok(self.value.clone())
    }

    /// Convert the atom to a JavaScript value (snake_case alias).
    #[napi(js_name = "to_value")]
    pub fn to_value_snake(&self) -> NapiResult<String> {
        self.to_value()
    }
}
