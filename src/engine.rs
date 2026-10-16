//! `JsEngine`: a JS-visible NAPI class that owns the N-API context handle (`napi::Engine`)
//! and the underlying QuickJS runtime (`rquickjs::Runtime`).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::*;
pub use napi::Env as Engine;
use napi_derive::napi;
use rquickjs::loader::{BuiltinLoader, BuiltinResolver, FileResolver, ModuleLoader, ScriptLoader};
use rquickjs::Runtime as QuickJSRuntime;

use crate::loader::Loader;

/// JS class wrapping N-API context (`napi::Env`) and QuickJS `Runtime`.
///
/// JS: `const engine = new Engine();`
#[napi(js_name = "Engine")]
pub struct JsEngine {
    pub(crate) env: Engine,
    pub(crate) runtime: QuickJSRuntime,
    pub(crate) disposed: Arc<AtomicBool>,
}

#[napi]
impl JsEngine {
    /// Capture the `napi::Env` that the runtime injects and create a QuickJS `Runtime`.
    #[napi(constructor)]
    pub fn new(env: napi::Env) -> Result<Self> {
        let disposed = Arc::new(AtomicBool::new(false));

        let hook_data = Arc::clone(&disposed);
        env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
            flag.store(true, Ordering::SeqCst);
        })?;

        let runtime = QuickJSRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(Self {
            env,
            runtime,
            disposed,
        })
    }

    /// Whether the owning addon (and thus this context) has been unloaded.
    #[napi]
    pub fn is_disposed(&self) -> bool {
        self.disposed.load(Ordering::SeqCst)
    }

    /// Set runtime info string.
    #[napi]
    pub fn set_info(&self, info: String) -> Result<()> {
        self.ensure_alive()?;
        self.runtime.set_info(info).map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Set memory limit in bytes.
    #[napi]
    pub fn set_memory_limit(&self, limit: u32) {
        self.runtime.set_memory_limit(limit as usize);
    }

    /// Set maximum stack size in bytes.
    #[napi]
    pub fn set_max_stack_size(&self, size: u32) {
        self.runtime.set_max_stack_size(size as usize);
    }

    /// Set memory threshold for garbage collection.
    #[napi]
    pub fn set_gc_threshold(&self, threshold: u32) {
        self.runtime.set_gc_threshold(threshold as usize);
    }

    /// Manually run QuickJS garbage collection.
    #[napi]
    pub fn run_gc(&self) {
        self.runtime.run_gc();
    }

    /// Test if any asynchronous jobs are pending in QuickJS runtime.
    #[napi]
    pub fn is_job_pending(&self) -> bool {
        self.runtime.is_job_pending()
    }

    /// Execute first pending job in QuickJS runtime.
    #[napi]
    pub fn execute_pending_job(&self) -> Result<bool> {
        self.runtime.execute_pending_job().map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Alias for execute_pending_job.
    #[napi]
    pub fn exec_pending_job(&self) -> Result<bool> {
        self.execute_pending_job()
    }

    /// Set the module loader configuration.
    #[napi]
    pub fn set_loader(&self, loader: &Loader) {
        let mut builtin_resolver = BuiltinResolver::default();
        let mut file_resolver = FileResolver::default();
        let mut builtin_loader = BuiltinLoader::default();

        for path in &loader.paths {
            file_resolver.add_path(path);
        }
        for (name, source) in &loader.modules {
            builtin_resolver.add_module(name.clone());
            builtin_loader.add_module(name.clone(), source.clone());
        }

        let resolver = (builtin_resolver, file_resolver);
        let loader_impl = (builtin_loader, ModuleLoader::default(), ScriptLoader::default());
        self.runtime.set_loader(resolver, loader_impl);
    }

    #[napi]
    pub fn create_array(&self, len: u32) -> Result<Array<'_>> {
        self.ensure_alive()?;
        let mut raw_val = std::ptr::null_mut();
        napi::check_status!(
            unsafe { napi::sys::napi_create_array_with_length(self.env.raw(), len as usize, &mut raw_val) },
            "Create array with length failed"
        )?;
        unsafe { Array::from_napi_value(self.env.raw(), raw_val) }
    }

    /// `Engine::create_string` — build a JS string.
    #[napi]
    pub fn create_string(&self, value: String) -> Result<Unknown<'_>> {
        self.ensure_alive()?;
        self.env.create_string(&value)?.into_unknown(&self.env)
    }

    /// `Engine::create_int32` — build a JS number from an `i32`.
    #[napi]
    pub fn create_int32(&self, int: i32) -> Result<Unknown<'_>> {
        self.ensure_alive()?;
        self.env.create_int32(int)?.into_unknown(&self.env)
    }

    /// `Engine::create_double` — build a JS number from an `f64`.
    #[napi]
    pub fn create_double(&self, value: f64) -> Result<Unknown<'_>> {
        self.ensure_alive()?;
        self.env.create_double(value)?.into_unknown(&self.env)
    }

    /// Does the global object have this property?
    #[napi]
    pub fn has_global(&self, name: String) -> Result<bool> {
        self.ensure_alive()?;
        let global = self.env.get_global()?;
        global.has_named_property(&name)
    }

    /// `Engine::get_napi_version` — the N-API version of the host runtime.
    #[napi]
    pub fn get_napi_version(&self) -> Result<u32> {
        self.ensure_alive()?;
        self.env.get_napi_version()
    }

    /// `Object::new` through the captured context.
    #[napi]
    pub fn make_object(&self) -> Result<Object<'_>> {
        self.ensure_alive()?;
        let mut obj = Object::new(&self.env)?;
        obj.set_named_property("ok", true)?;
        obj.set_named_property("engine", "napi")?;
        Ok(obj)
    }

    /// `Engine::set_instance_data` — persist VM-specific state on the Agent.
    #[napi]
    pub fn set_instance_data(&self, value: String) -> Result<()> {
        self.ensure_alive()?;
        self.env.set_instance_data(value, (), |_ctx| {})
    }

    /// `Engine::get_instance_data` — read back the persisted VM-specific state.
    #[napi]
    pub fn get_instance_data(&self) -> Result<Option<String>> {
        self.ensure_alive()?;
        Ok(self.env.get_instance_data::<String>()?.map(|s: &mut String| s.clone()))
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
