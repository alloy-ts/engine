//! NAPI bindings for `rquickjs::Runtime`.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::loader::{BuiltinLoader, BuiltinResolver, FileResolver, ScriptLoader};
use rquickjs::Runtime as QuickRuntime;

use crate::context::Context;
use crate::loader::Loader;

/// NAPI wrapper around `rquickjs::Runtime`.
#[napi(js_name = "Runtime")]
pub struct Runtime {
    pub(crate) inner: QuickRuntime,
}

#[napi]
impl Runtime {
    /// Create a new QuickJS runtime.
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        let inner = QuickRuntime::new().map_err(|e| {
            Error::new(Status::GenericFailure, format!("Failed to create QuickJS runtime: {e}"))
        })?;
        Ok(Self { inner })
    }

    /// Set a memory limit (in bytes) on the runtime.
    #[napi(js_name = "setMemoryLimit")]
    pub fn set_memory_limit(&self, limit: u32) {
        self.inner.set_memory_limit(limit as usize);
    }

    /// Set max stack size (in bytes) on the runtime.
    #[napi(js_name = "setMaxStackSize")]
    pub fn set_max_stack_size(&self, limit: u32) {
        self.inner.set_max_stack_size(limit as usize);
    }

    /// Set a memory threshold for garbage collection.
    #[napi(js_name = "setGcThreshold")]
    pub fn set_gc_threshold(&self, threshold: u32) {
        self.inner.set_gc_threshold(threshold as usize);
    }

    /// Manually run garbage collection.
    #[napi(js_name = "runGc")]
    pub fn run_gc(&self) {
        self.inner.run_gc();
    }

    /// Set the default module loader and resolver.
    #[napi(js_name = "setLoader")]
    pub fn set_loader(&self, _loader: &Loader) {
        let resolver = (BuiltinResolver::default(), FileResolver::default());
        let loader = (BuiltinLoader::default(), ScriptLoader::default());
        self.inner.set_loader(resolver, loader);
    }

    /// Check if any jobs are pending.
    #[napi(js_name = "isJobPending")]
    pub fn is_job_pending(&self) -> bool {
        self.inner.is_job_pending()
    }

    /// Execute the first pending job in the queue.
    #[napi(js_name = "executePendingJob")]
    pub fn execute_pending_job(&self) -> Result<bool> {
        self.inner.execute_pending_job().map_err(|e| {
            Error::new(Status::GenericFailure, format!("Pending job execution failed: {e}"))
        })
    }

    /// Create a new Context bound to this Runtime.
    #[napi(js_name = "createContext")]
    pub fn create_context(&self) -> Result<Context> {
        Context::full(&self.inner)
    }
}
