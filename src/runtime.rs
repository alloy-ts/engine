//! NAPI bindings for `rquickjs::Runtime`.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::Runtime;

#[napi(js_name = "Runtime")]
pub struct JsRuntime {
    pub(crate) inner: Runtime,
}

#[napi]
impl JsRuntime {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        let inner = Runtime::new().map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self { inner })
    }

    #[napi]
    pub fn set_info(&self, info: String) -> Result<()> {
        self.inner
            .set_info(info)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub fn set_memory_limit(&self, limit: u32) {
        self.inner.set_memory_limit(limit as usize);
    }

    #[napi]
    pub fn set_max_stack_size(&self, limit: u32) {
        self.inner.set_max_stack_size(limit as usize);
    }

    #[napi]
    pub fn set_gc_threshold(&self, threshold: u32) {
        self.inner.set_gc_threshold(threshold as usize);
    }

    #[napi]
    pub fn set_dump_flags(&self, flags: i64) {
        self.inner.set_dump_flags(flags as u64);
    }

    #[napi]
    pub fn run_gc(&self) {
        self.inner.run_gc();
    }

    #[napi]
    pub fn is_job_pending(&self) -> bool {
        self.inner.is_job_pending()
    }

    #[napi]
    pub fn execute_pending_job(&self) -> Result<bool> {
        self.inner
            .execute_pending_job()
            .map_err(|e| Error::from_reason(e.to_string()))
    }
}
