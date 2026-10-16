//! `Engine`: a JS-visible NAPI class that owns the N-API context handle and QuickJS Runtime capabilities.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::{JsGlobal, Unknown};
use napi_derive::napi;
use rquickjs::Runtime as QjsRuntime;

/// Re-export `napi::Env` as `Engine`.
pub use napi::Env as Engine;
/// Re-export `rquickjs::Runtime` as `Runtime`.
pub use rquickjs::Runtime;

/// JS class wrapping the N-API context (`napi::Env`) and QuickJS runtime capabilities.
///
/// JS: `const engine = new Engine();` — the context is injected by NAPI, so the
/// constructor takes no JavaScript arguments.
#[napi(js_name = "Engine")]
pub struct EngineClass {
  env: napi::Env,
  disposed: Arc<AtomicBool>,
  qjs_runtime: Option<QjsRuntime>,
}

#[napi]
impl EngineClass {
    /// Capture the `napi::Env` that the runtime injects and register a
    /// cleanup hook so we learn when it stops being valid.
    #[napi(constructor)]
    pub fn new(env: napi::Env) -> Result<Self> {
        let disposed = Arc::new(AtomicBool::new(false));

        // Share the flag with the cleanup hook: when the environment (and so the
        // addon instance) is torn down, mark this engine disposed.
        let hook_data = Arc::clone(&disposed);
        env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
            flag.store(true, Ordering::SeqCst);
        })?;

        let qjs_runtime = QjsRuntime::new().ok();

        Ok(Self {
          env,
          disposed,
          qjs_runtime,
        })
    }

    /// Whether the owning addon (and thus this context) has been unloaded.
    #[napi]
    pub fn is_disposed(&self) -> bool {
        self.disposed.load(Ordering::SeqCst)
    }

    #[napi]
    pub fn create_array(&self, len: u32) -> Result<Unknown<'_>> {
        self.ensure_alive()?;
        Ok(self.env.create_array(len)?.into_unknown(&self.env)?)
    }

    /// `Engine::create_string` — build a JS string.
    #[napi]
    pub fn create_string(&self, value: String) -> Result<Unknown<'_>> {
        self.ensure_alive()?;
        Ok(self.env.create_string(&value)?.into_unknown(&self.env)?)
    }

    /// `Engine::create_int32` — build a JS number from an `i32`.
    #[napi]
    pub fn create_int32(&self, int: i32) -> Result<Unknown<'_>> {
        self.ensure_alive()?;
        Ok(self.env.create_int32(int)?.into_unknown(&self.env)?)
    }

    /// `Engine::create_double` — build a JS number from an `f64`.
    #[napi]
    pub fn create_double(&self, value: f64) -> Result<Unknown<'_>> {
        self.ensure_alive()?;
        Ok(self.env.create_double(value)?.into_unknown(&self.env)?)
    }

    /// Resolve the JS global object.
    pub fn get_global(&self) -> Result<JsGlobal<'static>> {
        self.env.get_global()
    }

    /// Does the global object have this property?
    #[napi]
    pub fn has_global(&self, name: String) -> Result<bool> {
        self.ensure_alive()?;
        self.get_global()?.has_named_property(&name)
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
    ///
    /// The finalize callback runs when the Agent (addon instance) is torn down,
    /// i.e. the same unload event the cleanup hook observes.
    #[napi]
    pub fn set_instance_data(&self, value: String) -> Result<()> {
        self.ensure_alive()?;
        self.env.set_instance_data(value, (), |_ctx| {})
    }

    /// `Engine::get_instance_data` — read back the persisted VM-specific state.
    #[napi]
    pub fn get_instance_data(&self) -> Result<Option<String>> {
        self.ensure_alive()?;
        Ok(self.env.get_instance_data::<String>()?.cloned())
    }

    /// Sets the maximum memory limit in bytes for the QuickJS runtime.
    #[napi]
    pub fn set_memory_limit(&self, limit: u32) {
        if let Some(ref rt) = self.qjs_runtime {
            rt.set_memory_limit(limit as usize);
        }
    }

    /// Sets the maximum stack size in bytes for the QuickJS runtime.
    #[napi]
    pub fn set_max_stack_size(&self, limit: u32) {
        if let Some(ref rt) = self.qjs_runtime {
            rt.set_max_stack_size(limit as usize);
        }
    }

    /// Sets the memory threshold in bytes for triggering garbage collection.
    #[napi]
    pub fn set_gc_threshold(&self, threshold: u32) {
        if let Some(ref rt) = self.qjs_runtime {
            rt.set_gc_threshold(threshold as usize);
        }
    }

    /// Manually runs a cycle of cyclic garbage collection.
    #[napi]
    pub fn run_gc(&self) {
        if let Some(ref rt) = self.qjs_runtime {
            rt.run_gc();
        }
    }

    /// Checks whether at least one job is pending in the runtime queue.
    #[napi]
    pub fn is_job_pending(&self) -> bool {
        self.qjs_runtime.as_ref().map(|rt| rt.is_job_pending()).unwrap_or(false)
    }

    /// Executes the next pending job in the runtime queue.
    #[napi]
    pub fn execute_pending_job(&self) -> Result<bool> {
        if let Some(ref rt) = self.qjs_runtime {
            rt.execute_pending_job().map_err(|e| Error::from_reason(e.to_string()))
        } else {
            Ok(false)
        }
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
