//! `JsEngine`: a JS-visible NAPI class that owns the N-API context handle (`napi::Engine`) and QuickJS runtime.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::*;
pub use napi::Env as Engine;
use napi::Env;
use napi_derive::napi;
use rquickjs::Runtime as QuickRuntime;

/// JS class wrapping N-API context (`napi::Env`) and QuickJS runtime.
#[napi(js_name = "Engine")]
pub struct JsEngine {
    pub(crate) env: Env,
    pub(crate) disposed: Arc<AtomicBool>,
    pub(crate) quick_runtime: Option<QuickRuntime>,
}

#[napi]
impl JsEngine {
    /// Capture the `napi::Env` that the runtime injects and register a
    /// cleanup hook so we learn when it stops being valid.
    #[napi(constructor)]
    pub fn new(env: Env) -> Result<Self> {
        let disposed = Arc::new(AtomicBool::new(false));

        // Share the flag with the cleanup hook: when the environment (and so the
        // addon instance) is torn down, mark this engine disposed.
        let hook_data = Arc::clone(&disposed);
        env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
            flag.store(true, Ordering::SeqCst);
        })?;

        let quick_runtime = QuickRuntime::new().ok();

        Ok(Self {
            env,
            disposed,
            quick_runtime,
        })
    }

    /// Whether the owning addon (and thus this context) has been unloaded.
    #[napi]
    pub fn is_disposed(&self) -> bool {
        self.disposed.load(Ordering::SeqCst)
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

    /// `Engine::createString` — build a JS string.
    #[napi]
    pub fn create_string(&self, value: String) -> Result<Unknown<'_>> {
        self.ensure_alive()?;
        self.env.create_string(&value)?.into_unknown(&self.env)
    }

    /// `Engine::createInt32` — build a JS number from an `i32`.
    #[napi]
    pub fn create_int32(&self, int: i32) -> Result<Unknown<'_>> {
        self.ensure_alive()?;
        self.env.create_int32(int)?.into_unknown(&self.env)
    }

    /// `Engine::createDouble` — build a JS number from an `f64`.
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

    /// `Engine::getNapiVersion` — the N-API version of the host runtime.
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

    /// `Engine::setInstanceData` — persist VM-specific state on the Agent.
    #[napi]
    pub fn set_instance_data(&self, value: String) -> Result<()> {
        self.ensure_alive()?;
        self.env.set_instance_data(value, (), |_ctx| {})
    }

    /// `Engine::getInstanceData` — read back the persisted VM-specific state.
    #[napi]
    pub fn get_instance_data(&self) -> Result<Option<String>> {
        self.ensure_alive()?;
        Ok(self.env.get_instance_data::<String>()?.map(|s: &mut String| s.clone()))
    }

    /// QuickJS GC execution
    #[napi]
    pub fn run_gc(&self) -> Result<()> {
        self.ensure_alive()?;
        if let Some(rt) = &self.quick_runtime {
            rt.run_gc();
        }
        Ok(())
    }

    /// Set memory limit for QuickJS runtime
    #[napi]
    pub fn set_memory_limit(&self, limit: u32) -> Result<()> {
        self.ensure_alive()?;
        if let Some(rt) = &self.quick_runtime {
            rt.set_memory_limit(limit as usize);
        }
        Ok(())
    }

    /// Set max stack size for QuickJS runtime
    #[napi]
    pub fn set_max_stack_size(&self, limit: u32) -> Result<()> {
        self.ensure_alive()?;
        if let Some(rt) = &self.quick_runtime {
            rt.set_max_stack_size(limit as usize);
        }
        Ok(())
    }

    /// Set GC threshold for QuickJS runtime
    #[napi]
    pub fn set_gc_threshold(&self, threshold: u32) -> Result<()> {
        self.ensure_alive()?;
        if let Some(rt) = &self.quick_runtime {
            rt.set_gc_threshold(threshold as usize);
        }
        Ok(())
    }

    /// Test for pending jobs in QuickJS queue
    #[napi]
    pub fn is_job_pending(&self) -> bool {
        if let Some(rt) = &self.quick_runtime {
            rt.is_job_pending()
        } else {
            false
        }
    }

    /// Execute pending job in QuickJS queue
    #[napi]
    pub fn exec_pending_job(&self) -> Result<bool> {
        self.ensure_alive()?;
        if let Some(rt) = &self.quick_runtime {
            rt.execute_pending_job()
                .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
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
