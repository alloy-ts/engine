//! NAPI bindings for QuickJS Atom.

use napi::bindgen_prelude::Result as NapiResult;
use napi::Error;
use napi_derive::napi;
use rquickjs::{Atom as QjsAtom, Context as QjsContext, Runtime as QjsRuntime, Value};

fn to_napi_err(e: rquickjs::Error) -> Error {
    Error::from_reason(e.to_string())
}

/// A QuickJS Atom NAPI wrapper.
///
/// In QuickJS atoms are similar to interned strings but with additional uses.
/// A symbol for instance is just an atom.
#[napi(js_name = "Atom")]
pub struct Atom {
    _runtime: QjsRuntime,
    ctx: QjsContext,
    value: String,
}

impl Atom {
    fn create(str_val: String) -> Self {
        let runtime = QjsRuntime::new().expect("failed to create QuickJS runtime");
        let ctx = QjsContext::full(&runtime).expect("failed to create QuickJS context");
        ctx.with(|c| {
            let _atom = QjsAtom::from_str(c.clone(), &str_val);
        });
        Atom {
            _runtime: runtime,
            ctx,
            value: str_val,
        }
    }
}

#[napi]
impl Atom {
    #[napi(constructor)]
    pub fn new(value: Option<String>) -> Self {
        Self::create(value.unwrap_or_default())
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
            _runtime: runtime,
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
            _runtime: runtime,
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
            _runtime: runtime,
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
            _runtime: runtime,
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
            _runtime: runtime,
            ctx,
            value: str_val,
        }
    }

    /// Create an atom from a string.
    #[napi(factory)]
    pub fn from_str(name: String) -> Self {
        Self::create(name)
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
            _runtime: runtime,
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
    #[napi]
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

#[napi(object)]
pub struct InJsAtoms {
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
  #[napi(js_name = "Value")]
  pub value: String,
  #[napi(js_name = "Prototype")]
  pub prototype: String,
  #[napi(js_name = "Constructor")]
  pub constructor: String,
}

#[napi(object)]
pub struct InRsAtoms {
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

#[napi]
impl Atom {
  /// Predefined QuickJS atoms in JS.
  #[napi(getter, js_name = "inJs")]
  pub fn in_js() -> InJsAtoms {
    InJsAtoms {
      undefined: "undefined".to_string(),
      null: "null".to_string(),
      true_val: "true".to_string(),
      false_val: "false".to_string(),
      length: "length".to_string(),
      name: "name".to_string(),
      value: "value".to_string(),
      prototype: "prototype".to_string(),
      constructor: "constructor".to_string(),
    }
  }

  /// Native Rust primitive type names for atoms.
  #[napi(getter, js_name = "inRs")]
  pub fn in_rs() -> InRsAtoms {
    InRsAtoms {
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
}
