//! NAPI bindings for `rquickjs::Context`.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::Context;

use crate::engine::JsEngine;

#[napi(js_name = "Context")]
pub struct JsContext {
    pub(crate) inner: Context,
}

#[napi]
impl JsContext {
    #[napi(factory)]
    pub fn full(engine: &JsEngine) -> Result<Self> {
        let inner = Context::full(&engine.runtime).map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self { inner })
    }

    #[napi(factory)]
    pub fn base(engine: &JsEngine) -> Result<Self> {
        let inner = Context::base(&engine.runtime).map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self { inner })
    }

    #[napi]
    pub fn eval(&self, source: String) -> Result<String> {
        self.inner.with(|ctx| {
            let val: rquickjs::Value = ctx.eval(source).map_err(|e| Error::from_reason(e.to_string()))?;
            if let Some(s) = val.as_string() {
                s.to_string().map_err(|e| Error::from_reason(e.to_string()))
            } else if let Some(i) = val.as_int() {
                Ok(i.to_string())
            } else if let Some(f) = val.as_float() {
                Ok(f.to_string())
            } else if let Some(b) = val.as_bool() {
                Ok(b.to_string())
            } else {
                Ok(format!("{:?}", val))
            }
        })
    }
}
