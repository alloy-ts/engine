use napi::bindgen_prelude::Result as NapiResult;
use napi::Error;
use napi_derive::napi;
use rquickjs::{Atom as QjsAtom, Context as QjsContext, Runtime as QjsRuntime, Value};

fn to_napi_err(e: rquickjs::Error) -> Error {
    Error::from_reason(e.to_string())
}

/// In QuickJS atoms are similar to interned strings but with additional uses.
/// A symbol for instance is just an atom.
#[napi(object)]
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
            let js_val: Value = c.eval(val.as_str()).expect("failed eval");
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
    #[napi]
    pub fn to_value(&self) -> NapiResult<String> {
        Ok(self.value.clone())
    }
}
