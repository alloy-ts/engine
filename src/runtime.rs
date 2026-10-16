//! NAPI bindings for QuickJS Runtime.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::Runtime as QuickJSRuntime;

/// NAPI binding for QuickJS Runtime.
#[napi]
pub struct Runtime {
    pub(crate) inner: QuickJSRuntime,
}

#[napi]
impl Runtime {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        let inner = QuickJSRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Set runtime info string.
    #[napi]
    pub fn set_info(&self, info: String) -> Result<()> {
        self.inner.set_info(info).map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Set memory limit in bytes.
    #[napi]
    pub fn set_memory_limit(&self, limit: u32) {
        self.inner.set_memory_limit(limit as usize);
    }

    /// Set maximum stack size in bytes.
    #[napi]
    pub fn set_max_stack_size(&self, size: u32) {
        self.inner.set_max_stack_size(size as usize);
    }

    /// Set memory threshold for garbage collection.
    #[napi]
    pub fn set_gc_threshold(&self, threshold: u32) {
        self.inner.set_gc_threshold(threshold as usize);
    }

    /// Run the QuickJS garbage collector.
    #[napi]
    pub fn run_gc(&self) {
        self.inner.run_gc();
    }

    /// Test if any asynchronous jobs are pending.
    #[napi]
    pub fn is_job_pending(&self) -> bool {
        self.inner.is_job_pending()
    }

    /// Execute the first pending job in the queue.
    #[napi]
    pub fn execute_pending_job(&self) -> Result<bool> {
        self.inner.execute_pending_job().map_err(|e| Error::from_reason(e.to_string()))
    }
}

impl Clone for Runtime {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
