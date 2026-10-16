//! NAPI bindings for QuickJS [`Atom`].

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::{Context as QjsContext, Runtime as QjsRuntime};

/// JS class wrapping a QuickJS [`Atom`].
#[napi(js_name = "Atom")]
pub struct Atom {
    value: String,
}

#[napi]
impl Atom {
    /// Create an Atom from a string.
    #[napi(factory)]
    pub fn from_str(name: String) -> Result<Self> {
        let rt = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
        let ctx = QjsContext::full(&rt).map_err(|e| Error::from_reason(e.to_string()))?;
        ctx.with(|ctx| {
            let atom = rquickjs::Atom::from_str(ctx, &name)
                .map_err(|e| Error::from_reason(e.to_string()))?;
            let s = atom.to_string().map_err(|e| Error::from_reason(e.to_string()))?;
            Ok(Self { value: s })
        })
    }

    /// Create an Atom from a u32.
    #[napi(factory)]
    pub fn from_u32(val: u32) -> Result<Self> {
        let rt = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
        let ctx = QjsContext::full(&rt).map_err(|e| Error::from_reason(e.to_string()))?;
        ctx.with(|ctx| {
            let atom = rquickjs::Atom::from_u32(ctx, val)
                .map_err(|e| Error::from_reason(e.to_string()))?;
            let s = atom.to_string().map_err(|e| Error::from_reason(e.to_string()))?;
            Ok(Self { value: s })
        })
    }

    /// Create an Atom from an i32.
    #[napi(factory)]
    pub fn from_i32(val: i32) -> Result<Self> {
        let rt = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
        let ctx = QjsContext::full(&rt).map_err(|e| Error::from_reason(e.to_string()))?;
        ctx.with(|ctx| {
            let atom = rquickjs::Atom::from_i32(ctx, val)
                .map_err(|e| Error::from_reason(e.to_string()))?;
            let s = atom.to_string().map_err(|e| Error::from_reason(e.to_string()))?;
            Ok(Self { value: s })
        })
    }

    /// Create an Atom from a bool.
    #[napi(factory)]
    pub fn from_bool(val: bool) -> Result<Self> {
        let rt = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
        let ctx = QjsContext::full(&rt).map_err(|e| Error::from_reason(e.to_string()))?;
        ctx.with(|ctx| {
            let atom = rquickjs::Atom::from_bool(ctx, val)
                .map_err(|e| Error::from_reason(e.to_string()))?;
            let s = atom.to_string().map_err(|e| Error::from_reason(e.to_string()))?;
            Ok(Self { value: s })
        })
    }

    /// Create an Atom from a f64.
    #[napi(factory)]
    pub fn from_f64(val: f64) -> Result<Self> {
        let rt = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
        let ctx = QjsContext::full(&rt).map_err(|e| Error::from_reason(e.to_string()))?;
        ctx.with(|ctx| {
            let atom = rquickjs::Atom::from_f64(ctx, val)
                .map_err(|e| Error::from_reason(e.to_string()))?;
            let s = atom.to_string().map_err(|e| Error::from_reason(e.to_string()))?;
            Ok(Self { value: s })
        })
    }

    /// Convert the atom to string representation.
    #[napi]
    pub fn to_string(&self) -> String {
        self.value.clone()
    }
}
