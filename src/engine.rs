//! `JsEngine`: a JS-visible NAPI class that owns the N-API context handle (`napi::Engine`)
//! and the QuickJS [`Runtime`].

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::*;
pub use napi::Env as Engine;
use napi_derive::napi;
use rquickjs::Runtime as QjsRuntime;

/// JS class wrapping the N-API context (`napi::Env`) and QuickJS [`Runtime`].
///
/// JS: `const engine = new Engine();` — the context is injected by NAPI, so the
/// constructor takes no JavaScript arguments.
#[napi(js_name = "Engine")]
pub struct JsEngine {
    pub(crate) env: Env,
    pub(crate) disposed: Arc<AtomicBool>,
    pub(crate) qjs_rt: Option<QjsRuntime>,
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

        let qjs_rt = QjsRuntime::new().ok();

        Ok(Self {
            env,
            disposed,
            qjs_rt,
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
        Ok(self.env.get_instance_data::<String>()?.map(|s: &mut String| s.clone()))
    }

    // --- QuickJS Runtime methods merged into Engine ---

    /// Set runtime info string.
    #[napi]
    pub fn set_info(&self, info: String) -> Result<()> {
        if let Some(rt) = &self.qjs_rt {
            rt.set_info(info).map_err(|e| Error::from_reason(e.to_string()))?;
        }
        Ok(())
    }

    /// Set a limit on the max amount of memory (in bytes) the runtime will use.
    #[napi]
    pub fn set_memory_limit(&self, limit: u32) {
        if let Some(rt) = &self.qjs_rt {
            rt.set_memory_limit(limit as usize);
        }
    }

    /// Set a limit on the max size of stack (in bytes) the runtime will use.
    #[napi]
    pub fn set_max_stack_size(&self, limit: u32) {
        if let Some(rt) = &self.qjs_rt {
            rt.set_max_stack_size(limit as usize);
        }
    }

    /// Set a memory threshold for garbage collection.
    #[napi]
    pub fn set_gc_threshold(&self, threshold: u32) {
        if let Some(rt) = &self.qjs_rt {
            rt.set_gc_threshold(threshold as usize);
        }
    }

    /// Set debug flags for dumping memory.
    #[napi]
    pub fn set_dump_flags(&self, flags: u32) {
        if let Some(rt) = &self.qjs_rt {
            rt.set_dump_flags(flags as u64);
        }
    }

    /// Manually run garbage collection.
    #[napi]
    pub fn run_gc(&self) {
        if let Some(rt) = &self.qjs_rt {
            rt.run_gc();
        }
    }

    /// Check whether there are pending jobs in the QuickJS job queue.
    #[napi]
    pub fn is_job_pending(&self) -> bool {
        self.qjs_rt.as_ref().map_or(false, |rt| rt.is_job_pending())
    }

    /// Execute the first pending job in the job queue.
    #[napi]
    pub fn execute_pending_job(&self) -> Result<bool> {
        if let Some(rt) = &self.qjs_rt {
            rt.execute_pending_job()
                .map_err(|_e| Error::from_reason("Pending job threw exception"))
        } else {
            Ok(false)
        }
    }

    /// Set a module loader and resolver on this runtime.
    #[napi]
    pub fn set_loader(
        &self,
        resolver: &mut crate::loader::BuiltinResolverWrapper,
        loader: &mut crate::loader::BuiltinLoaderWrapper,
    ) {
        if let Some(rt) = &self.qjs_rt {
            if let (Some(r), Some(l)) = (resolver.inner.take(), loader.inner.take()) {
                rt.set_loader(r, l);
            }
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
