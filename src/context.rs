//! NAPI bindings for QuickJS Context.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::Context as QuickJSContext;

use crate::engine::JsEngine;

/// NAPI binding for QuickJS Context.
#[napi]
#[derive(Clone)]
pub struct Context {
    pub(crate) inner: QuickJSContext,
}

#[napi]
impl Context {
    /// Creates a base context with only the required functions registered.
    #[napi(factory)]
    pub fn base(engine: &JsEngine) -> Result<Self> {
        let inner = QuickJSContext::base(&engine.runtime)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Creates a context with all standard available intrinsics registered.
    #[napi(factory)]
    pub fn full(engine: &JsEngine) -> Result<Self> {
        let inner = QuickJSContext::full(&engine.runtime)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Evaluates JavaScript code in the context and returns the result as a string.
    #[napi]
    pub fn eval(&self, code: String) -> Result<String> {
        self.inner.with(|ctx| {
            let val: rquickjs::Value = ctx.eval(code.as_str()).map_err(|e| Error::from_reason(e.to_string()))?;
            if val.is_undefined() {
                Ok("undefined".to_string())
            } else if val.is_null() {
                Ok("null".to_string())
            } else if let Some(s) = val.as_string() {
                s.to_string().map_err(|e| Error::from_reason(e.to_string()))
            } else if let Some(b) = val.as_bool() {
                Ok(b.to_string())
            } else if let Some(i) = val.as_int() {
                Ok(i.to_string())
            } else if let Some(f) = val.as_float() {
                Ok(f.to_string())
            } else if let Ok(json) = ctx.json_stringify(&val) {
                if let Some(json_str) = json {
                    json_str.to_string().map_err(|e| Error::from_reason(e.to_string()))
                } else {
                    Ok("[object Object]".to_string())
                }
            } else {
                Ok("[object Object]".to_string())
            }
        })
    }

    /// Evaluates JavaScript code from a file.
    #[napi]
    pub fn eval_file(&self, path: String) -> Result<String> {
        self.inner.with(|ctx| {
            let val: rquickjs::Value = ctx.eval_file(path.as_str()).map_err(|e| Error::from_reason(e.to_string()))?;
            if let Ok(json) = ctx.json_stringify(&val) {
                if let Some(s) = json {
                    return s.to_string().map_err(|e| Error::from_reason(e.to_string()));
                }
            }
            Ok("undefined".to_string())
        })
    }

    /// Parses JSON into JavaScript value and returns stringified result.
    #[napi]
    pub fn json_parse(&self, json: String) -> Result<String> {
        self.inner.with(|ctx| {
            let val = ctx.json_parse(json).map_err(|e| Error::from_reason(e.to_string()))?;
            if let Ok(Some(s)) = ctx.json_stringify(val) {
                s.to_string().map_err(|e| Error::from_reason(e.to_string()))
            } else {
                Ok("null".to_string())
            }
        })
    }

    /// Stringifies JavaScript value represented as string JSON.
    #[napi]
    pub fn json_stringify(&self, value: String) -> Result<String> {
        self.inner.with(|ctx| {
            let val = ctx.eval::<rquickjs::Value, _>(value.as_str()).map_err(|e| Error::from_reason(e.to_string()))?;
            if let Ok(Some(s)) = ctx.json_stringify(val) {
                s.to_string().map_err(|e| Error::from_reason(e.to_string()))
            } else {
                Ok("undefined".to_string())
            }
        })
    }
}
