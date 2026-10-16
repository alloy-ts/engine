//! NAPI bindings for QuickJS [`Runtime`].

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::Runtime as QjsRuntime;

/// JS class wrapping a QuickJS [`Runtime`].
#[napi(js_name = "Runtime")]
pub struct Runtime {
    pub(crate) inner: QjsRuntime,
}

#[napi]
impl Runtime {
    /// Create a new QuickJS runtime instance.
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        let rt = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self { inner: rt })
    }

    /// Set runtime info string.
    #[napi]
    pub fn set_info(&self, info: String) -> Result<()> {
        self.inner.set_info(info).map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Set a limit on the max amount of memory (in bytes) the runtime will use.
    #[napi]
    pub fn set_memory_limit(&self, limit: u32) {
        self.inner.set_memory_limit(limit as usize);
    }

    /// Set a limit on the max size of stack (in bytes) the runtime will use.
    #[napi]
    pub fn set_max_stack_size(&self, limit: u32) {
        self.inner.set_max_stack_size(limit as usize);
    }

    /// Set a memory threshold for garbage collection.
    #[napi]
    pub fn set_gc_threshold(&self, threshold: u32) {
        self.inner.set_gc_threshold(threshold as usize);
    }

    /// Set debug flags for dumping memory.
    #[napi]
    pub fn set_dump_flags(&self, flags: u32) {
        self.inner.set_dump_flags(flags as u64);
    }

    /// Manually run garbage collection.
    #[napi]
    pub fn run_gc(&self) {
        self.inner.run_gc();
    }

    /// Check whether there are pending jobs in the QuickJS job queue.
    #[napi]
    pub fn is_job_pending(&self) -> bool {
        self.inner.is_job_pending()
    }

    /// Execute the first pending job in the job queue.
    #[napi]
    pub fn execute_pending_job(&self) -> Result<bool> {
        self.inner
            .execute_pending_job()
            .map_err(|_e| Error::from_reason("Pending job threw exception"))
    }
}
