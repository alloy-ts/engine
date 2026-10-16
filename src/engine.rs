//! `JsEngine`: JS-visible NAPI class wrapping N-API context (`napi::Env`) and QuickJS runtime bindings.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::Env;
use napi_derive::napi;

use rquickjs::loader::{BuiltinLoader, BuiltinResolver, FileResolver, ScriptLoader};
use rquickjs::Runtime as QuickRuntime;

use crate::context::Context;
use crate::loader::Loader;

pub use napi::Env as Engine;

/// JS class wrapping N-API context (`napi::Env`) and QuickJS runtime.
#[napi(js_name = "Engine")]
pub struct JsEngine {
    pub(crate) env: Env,
    pub(crate) disposed: Arc<AtomicBool>,
    pub(crate) runtime: QuickRuntime,
}

#[napi]
impl JsEngine {
    /// Capture the `napi::Env` that the runtime injects and create a QuickJS runtime instance.
    #[napi(constructor)]
    pub fn new(env: Env) -> Result<Self> {
        let disposed = Arc::new(AtomicBool::new(false));

        // Share the flag with the cleanup hook: when the environment (and so the
        // addon instance) is torn down, mark this engine disposed.
        let hook_data = Arc::clone(&disposed);
        env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
            flag.store(true, Ordering::SeqCst);
        })?;

        let runtime = QuickRuntime::new().map_err(|e| {
            Error::new(Status::GenericFailure, format!("Failed to create QuickJS runtime: {e}"))
        })?;

        Ok(Self {
            env,
            disposed,
            runtime,
        })
    }

    /// Whether the owning addon (and thus this context) has been unloaded.
    #[napi(js_name = "isDisposed")]
    pub fn is_disposed(&self) -> bool {
        self.disposed.load(Ordering::SeqCst)
    }

    #[napi(js_name = "createArray")]
    pub fn create_array(&self, len: u32) -> Result<Array<'_>> {
        self.ensure_alive()?;
        let mut raw_val = std::ptr::null_mut();
        napi::check_status!(
            unsafe { napi::sys::napi_create_array_with_length(self.env.raw(), len as usize, &mut raw_val) },
            "Create array with length failed"
        )?;
        unsafe { Array::from_napi_value(self.env.raw(), raw_val) }
    }

    /// `Engine::createString` — build a JS string.
    #[napi(js_name = "createString")]
    pub fn create_string(&self, value: String) -> Result<Unknown<'_>> {
        self.ensure_alive()?;
        self.env.create_string(&value)?.into_unknown(&self.env)
    }

    /// `Engine::createInt32` — build a JS number from an `i32`.
    #[napi(js_name = "createInt32")]
    pub fn create_int32(&self, int: i32) -> Result<Unknown<'_>> {
        self.ensure_alive()?;
        self.env.create_int32(int)?.into_unknown(&self.env)
    }

    /// `Engine::createDouble` — build a JS number from an `f64`.
    #[napi(js_name = "createDouble")]
    pub fn create_double(&self, value: f64) -> Result<Unknown<'_>> {
        self.ensure_alive()?;
        self.env.create_double(value)?.into_unknown(&self.env)
    }

    /// Does the global object have this property?
    #[napi(js_name = "hasGlobal")]
    pub fn has_global(&self, name: String) -> Result<bool> {
        self.ensure_alive()?;
        let global = self.env.get_global()?;
        global.has_named_property(&name)
    }

    /// `Engine::getNapiVersion` — the N-API version of the host runtime.
    #[napi(js_name = "getNapiVersion")]
    pub fn get_napi_version(&self) -> Result<u32> {
        self.ensure_alive()?;
        self.env.get_napi_version()
    }

    /// `Object::new` through the captured context.
    #[napi(js_name = "makeObject")]
    pub fn make_object(&self) -> Result<Object<'_>> {
        self.ensure_alive()?;
        let mut obj = Object::new(&self.env)?;
        obj.set_named_property("ok", true)?;
        obj.set_named_property("engine", "napi")?;
        Ok(obj)
    }

    /// `Engine::setInstanceData` — persist VM-specific state on the Agent.
    #[napi(js_name = "setInstanceData")]
    pub fn set_instance_data(&self, value: String) -> Result<()> {
        self.ensure_alive()?;
        self.env.set_instance_data(value, (), |_ctx| {})
    }

    /// `Engine::getInstanceData` — read back the persisted VM-specific state.
    #[napi(js_name = "getInstanceData")]
    pub fn get_instance_data(&self) -> Result<Option<String>> {
        self.ensure_alive()?;
        Ok(self.env.get_instance_data::<String>()?.map(|s: &mut String| s.clone()))
    }

    // --- QuickJS Runtime methods ---

    /// Set a memory limit (in bytes) on the runtime.
    #[napi(js_name = "setMemoryLimit")]
    pub fn set_memory_limit(&self, limit: u32) {
        self.runtime.set_memory_limit(limit as usize);
    }

    /// Set max stack size (in bytes) on the runtime.
    #[napi(js_name = "setMaxStackSize")]
    pub fn set_max_stack_size(&self, limit: u32) {
        self.runtime.set_max_stack_size(limit as usize);
    }

    /// Set a memory threshold for garbage collection.
    #[napi(js_name = "setGcThreshold")]
    pub fn set_gc_threshold(&self, threshold: u32) {
        self.runtime.set_gc_threshold(threshold as usize);
    }

    /// Manually run garbage collection.
    #[napi(js_name = "runGc")]
    pub fn run_gc(&self) {
        self.runtime.run_gc();
    }

    /// Set module loader and resolver.
    #[napi(js_name = "setLoader")]
    pub fn set_loader(&self, _loader: &Loader) {
        let resolver = (BuiltinResolver::default(), FileResolver::default());
        let loader = (BuiltinLoader::default(), ScriptLoader::default());
        self.runtime.set_loader(resolver, loader);
    }

    /// Check if any jobs are pending.
    #[napi(js_name = "isJobPending")]
    pub fn is_job_pending(&self) -> bool {
        self.runtime.is_job_pending()
    }

    /// Execute the first pending job in the queue.
    #[napi(js_name = "executePendingJob")]
    pub fn execute_pending_job(&self) -> Result<bool> {
        self.runtime.execute_pending_job().map_err(|e| {
            Error::new(Status::GenericFailure, format!("Pending job execution failed: {e}"))
        })
    }

    /// Alias for executePendingJob (`engine.execPendingJob`).
    #[napi(js_name = "execPendingJob")]
    pub fn exec_pending_job(&self) -> Result<bool> {
        self.execute_pending_job()
    }

    /// Create a new Context bound to this Engine runtime.
    #[napi(js_name = "createContext")]
    pub fn create_context(&self) -> Result<Context> {
        Context::full(&self.runtime)
    }

    /// Guard: refuse to use a context that belongs to an unloaded addon.
    fn ensure_alive(&self) -> Result<()> {
        if self.disposed.load(Ordering::SeqCst) {
            return Err(Error::new(
                Status::GenericFailure,
                "Engine: N-API context has been disposed (addon unloaded)",
            ));
        }
        Ok(())
    }
}
