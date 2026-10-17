use napi::bindgen_prelude::Result as NapiResult;
use napi::Error;
use napi_derive::napi;
use rquickjs::{Atom as QjsAtom, Context as QjsContext, Runtime as QjsRuntime, Value};

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
    #[napi(factory, js_name = "from_predefined")]
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
    #[napi(factory, js_name = "from_u32")]
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
    #[napi(factory, js_name = "from_i32")]
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

    /// Create an atom from a `u64`.
    #[napi(factory, js_name = "from_u64")]
    pub fn from_u64(val: f64) -> Self {
        let u = val as u64;
        if u <= u32::MAX as u64 {
            Self::from_u32(u as u32)
        } else {
            Self::from_str(u.to_string())
        }
    }

    /// Create an atom from an `i64`.
    #[napi(factory, js_name = "from_i64")]
    pub fn from_i64(val: i64) -> Self {
        if val >= i32::MIN as i64 && val <= i32::MAX as i64 {
            Self::from_i32(val as i32)
        } else {
            Self::from_str(val.to_string())
        }
    }

    /// Create an atom from a `bool`.
    #[napi(factory, js_name = "from_bool")]
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
    #[napi(factory, js_name = "from_f64")]
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

    /// Create an atom from a `f32`.
    #[napi(factory, js_name = "from_f32")]
    pub fn from_f32(val: f64) -> Self {
        Self::from_f64(val)
    }

    /// Create an atom from a string.
    #[napi(factory, js_name = "from_str")]
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
    #[napi(factory, js_name = "from_value")]
    pub fn from_value(val: String) -> Self {
        let runtime = QjsRuntime::new().expect("failed to create QuickJS runtime");
        let ctx = QjsContext::full(&runtime).expect("failed to create QuickJS context");
        let str_val = ctx.with(|c| {
            let js_val: Value = c.eval(val.as_str()).unwrap_or_else(|_| {
                c.eval(format!("{:?}", val).as_str()).expect("failed eval fallback")
            });
            let atom = QjsAtom::from_value(c.clone(), &js_val).unwrap_or_else(|_| {
                QjsAtom::from_str(c.clone(), &val).expect("failed atom fallback")
            });
            atom.to_string().expect("failed to stringify atom")
        });
        Atom {
            ctx,
            value: str_val,
        }
    }

    /// Static factory method for native U32 primitive atom creation.
    #[napi(factory, js_name = "U32")]
    pub fn u32_primitive(val: u32) -> Self {
        Self::from_u32(val)
    }

    /// Static factory method for native I32 primitive atom creation.
    #[napi(factory, js_name = "I32")]
    pub fn i32_primitive(val: i32) -> Self {
        Self::from_i32(val)
    }

    /// Static factory method for native U64 primitive atom creation.
    #[napi(factory, js_name = "U64")]
    pub fn u64_primitive(val: f64) -> Self {
        Self::from_u64(val)
    }

    /// Static factory method for native I64 primitive atom creation.
    #[napi(factory, js_name = "I64")]
    pub fn i64_primitive(val: i64) -> Self {
        Self::from_i64(val)
    }

    /// Static factory method for native F64 primitive atom creation.
    #[napi(factory, js_name = "F64")]
    pub fn f64_primitive(val: f64) -> Self {
        Self::from_f64(val)
    }

    /// Static factory method for native F32 primitive atom creation.
    #[napi(factory, js_name = "F32")]
    pub fn f32_primitive(val: f64) -> Self {
        Self::from_f32(val)
    }

    /// Static factory method for native BOOL primitive atom creation.
    #[napi(factory, js_name = "BOOL")]
    pub fn bool_primitive(val: bool) -> Self {
        Self::from_bool(val)
    }

    /// Static factory method for native STRING primitive atom creation.
    #[napi(factory, js_name = "STRING")]
    pub fn string_primitive(name: String) -> Self {
        Self::from_str(name)
    }

    /// Convert the atom to a JavaScript string.
    #[napi(js_name = "to_string")]
    pub fn to_string(&self) -> NapiResult<String> {
        self.ctx.with(|c| {
            let atom = QjsAtom::from_str(c.clone(), &self.value).map_err(to_napi_err)?;
            atom.to_string().map_err(to_napi_err)
        })
    }

    /// Convert the atom to a JavaScript string representation.
    #[napi(js_name = "to_js_string")]
    pub fn to_js_string(&self) -> NapiResult<String> {
        self.ctx.with(|c| {
            let atom = QjsAtom::from_str(c.clone(), &self.value).map_err(to_napi_err)?;
            let js_str = atom.to_js_string().map_err(to_napi_err)?;
            js_str.to_string().map_err(to_napi_err)
        })
    }

    /// Convert the atom to a JavaScript value.
    #[napi(js_name = "to_value")]
    pub fn to_value(&self) -> NapiResult<String> {
        Ok(self.value.clone())
    }
}
