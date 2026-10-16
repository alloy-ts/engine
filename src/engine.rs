use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::Env;
use napi_derive::napi;
use rquickjs::{
    runtime::WeakRuntime as QjsWeakRuntime, Context as QjsContext, Runtime as QjsRuntime,
};
use serde_json::json;

use crate::context::JsContext;
use crate::loader::JsLoader;

/// Map a [`rquickjs::Error`] into a `napi::Error` so JS sees a thrown exception.
fn to_napi_err(e: rquickjs::Error) -> Error {
    Error::from_reason(e.to_string())
}

/// JS class wrapping both the host N-API context (`napi::Env`) and an embedded
/// QuickJS [`Runtime`](rquickjs::Runtime) engine.
///
/// JS: `const engine = new Engine();`
#[napi(js_name = "Engine")]
pub struct JsEngine {
    pub(crate) env: Env,
    pub(crate) disposed: Arc<AtomicBool>,
    inner: QjsRuntime,
}

#[napi]
impl JsEngine {
    /// Capture the `napi::Env` that the runtime injects and create an embedded
    /// QuickJS runtime. Accepts an optional module `Loader`.
    #[napi(constructor)]
    pub fn new(env: Env, loader: Option<&JsLoader>) -> Result<Self> {
        let disposed = Arc::new(AtomicBool::new(false));

        let hook_data = Arc::clone(&disposed);
        env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
            flag.store(true, Ordering::SeqCst);
        })?;

        let inner = QjsRuntime::new().map_err(to_napi_err)?;
        if let Some(loader) = loader {
            inner.set_loader(loader.resolver(), loader.loader());
        }

        Ok(Self {
            env,
            disposed,
            inner,
        })
    }

    /// Whether the owning addon (and thus this N-API context) has been unloaded.
    #[napi(js_name = "is_disposed")]
    pub fn is_disposed(&self) -> bool {
        self.disposed.load(Ordering::SeqCst)
    }

    /// `Engine::create_array` — build a JS array via N-API.
    #[napi(js_name = "create_array")]
    pub fn create_array(&self, len: u32) -> Result<Array<'_>> {
        self.ensure_alive()?;
        let mut raw_val = std::ptr::null_mut();
        napi::check_status!(
            unsafe { napi::sys::napi_create_array_with_length(self.env.raw(), len as usize, &mut raw_val) },
            "Create array with length failed"
        )?;
        unsafe { Array::from_napi_value(self.env.raw(), raw_val) }
    }

    /// `Engine::create_string` — build a JS string via N-API.
    #[napi(js_name = "create_string")]
    pub fn create_string(&self, value: String) -> Result<Unknown<'_>> {
        self.ensure_alive()?;
        self.env.create_string(&value)?.into_unknown(&self.env)
    }

    /// `Engine::create_int32` — build a JS number from an `i32` via N-API.
    #[napi(js_name = "create_int32")]
    pub fn create_int32(&self, int: i32) -> Result<Unknown<'_>> {
        self.ensure_alive()?;
        self.env.create_int32(int)?.into_unknown(&self.env)
    }

    /// `Engine::create_double` — build a JS number from an `f64` via N-API.
    #[napi(js_name = "create_double")]
    pub fn create_double(&self, value: f64) -> Result<Unknown<'_>> {
        self.ensure_alive()?;
        self.env.create_double(value)?.into_unknown(&self.env)
    }

    /// Does the global object have this property?
    #[napi(js_name = "has_global")]
    pub fn has_global(&self, name: String) -> Result<bool> {
        self.ensure_alive()?;
        let global = self.env.get_global()?;
        global.has_named_property(&name)
    }

    /// `Engine::get_napi_version` — the N-API version of the host runtime.
    #[napi(js_name = "get_napi_version")]
    pub fn get_napi_version(&self) -> Result<u32> {
        self.ensure_alive()?;
        self.env.get_napi_version()
    }

    /// `Object::new` through the captured N-API context.
    #[napi(js_name = "make_object")]
    pub fn make_object(&self) -> Result<Object<'_>> {
        self.ensure_alive()?;
        let mut obj = Object::new(&self.env)?;
        obj.set_named_property("ok", true)?;
        obj.set_named_property("engine", "napi")?;
        Ok(obj)
    }

    /// `Engine::set_instance_data` — persist VM-specific state on the Agent.
    #[napi(js_name = "set_instance_data")]
    pub fn set_instance_data(&self, value: String) -> Result<()> {
        self.ensure_alive()?;
        self.env.set_instance_data(value, (), |_ctx| {})
    }

    /// `Engine::get_instance_data` — read back the persisted VM-specific state.
    #[napi(js_name = "get_instance_data")]
    pub fn get_instance_data(&self) -> Result<Option<String>> {
        self.ensure_alive()?;
        Ok(self.env.get_instance_data::<String>()?.map(|s: &mut String| s.clone()))
    }

    /// Attach (or replace) the QuickJS module loader/resolver for this engine.
    #[napi(js_name = "set_loader")]
    pub fn set_loader(&self, loader: &JsLoader) -> Result<()> {
        self.inner.set_loader(loader.resolver(), loader.loader());
        Ok(())
    }

    /// Create a QuickJS `Context` sharing this engine.
    #[napi(js_name = "create_context")]
    pub fn create_context(&self) -> Result<JsContext> {
        let inner = QjsContext::full(&self.inner).map_err(to_napi_err)?;
        Ok(JsContext::from_inner(inner))
    }

    /// Get a weak handle to this engine.
    #[napi]
    pub fn weak(&self) -> JsWeakEngine {
        JsWeakEngine {
            inner: self.inner.weak(),
        }
    }

    /// Set a limit (in bytes) on the max amount of memory the runtime will use.
    #[napi(js_name = "set_memory_limit")]
    pub fn set_memory_limit(&self, limit: f64) -> Result<()> {
        self.inner.set_memory_limit(limit as usize);
        Ok(())
    }

    /// Set a limit (in bytes) on the max stack size the runtime will use.
    #[napi(js_name = "set_max_stack_size")]
    pub fn set_max_stack_size(&self, limit: f64) -> Result<()> {
        self.inner.set_max_stack_size(limit as usize);
        Ok(())
    }

    /// Set the memory threshold (in bytes) that triggers garbage collection.
    #[napi(js_name = "set_gc_threshold")]
    pub fn set_gc_threshold(&self, threshold: f64) -> Result<()> {
        self.inner.set_gc_threshold(threshold as usize);
        Ok(())
    }

    /// Set the runtime's informational string.
    #[napi(js_name = "set_info")]
    pub fn set_info(&self, info: String) -> Result<()> {
        self.inner.set_info(info).map_err(to_napi_err)
    }

    /// Manually run garbage collection on the QuickJS engine.
    #[napi(js_name = "run_gc")]
    pub fn run_gc(&self) -> Result<()> {
        self.inner.run_gc();
        Ok(())
    }

    /// Memory-usage statistics as a plain object.
    #[napi(js_name = "memory_usage")]
    pub fn memory_usage(&self) -> Result<serde_json::Value> {
        let u = self.inner.memory_usage();
        Ok(json!({
            "malloc_size": u.malloc_size,
            "malloc_limit": u.malloc_limit,
            "memory_used_size": u.memory_used_size,
            "malloc_count": u.malloc_count,
            "memory_used_count": u.memory_used_count,
            "atom_count": u.atom_count,
            "atom_size": u.atom_size,
            "str_count": u.str_count,
            "str_size": u.str_size,
            "obj_count": u.obj_count,
            "obj_size": u.obj_size,
            "prop_count": u.prop_count,
            "prop_size": u.prop_size,
            "shape_count": u.shape_count,
            "shape_size": u.shape_size,
            "js_func_count": u.js_func_count,
            "js_func_size": u.js_func_size,
            "js_func_code_size": u.js_func_code_size,
            "js_func_pc2line_count": u.js_func_pc2line_count,
            "js_func_pc2line_size": u.js_func_pc2line_size,
            "c_func_count": u.c_func_count,
            "array_count": u.array_count,
            "fast_array_count": u.fast_array_count,
            "fast_array_elements": u.fast_array_elements,
            "binary_object_count": u.binary_object_count,
            "binary_object_size": u.binary_object_size,
        }))
    }

    /// Whether at least one QuickJS job is pending.
    #[napi(js_name = "is_job_pending")]
    pub fn is_job_pending(&self) -> Result<bool> {
        Ok(self.inner.is_job_pending())
    }

    /// Execute the first pending job.
    #[napi(js_name = "execute_pending_job")]
    pub fn execute_pending_job(&self) -> Result<bool> {
        self.inner.execute_pending_job().map_err(|job_ex| {
            let msg = job_ex.0.with(|ctx| {
                let thrown = ctx.catch();
                ctx.json_stringify(thrown)
                    .ok()
                    .flatten()
                    .and_then(|s| s.to_string().ok())
                    .unwrap_or_else(|| "unknown job exception".to_string())
            });
            Error::from_reason(format!("job exception: {msg}"))
        })
    }

    /// Alias for `execute_pending_job` as `execPendingJob`.
    #[napi(js_name = "execPendingJob")]
    pub fn exec_pending_job(&self) -> Result<bool> {
        self.execute_pending_job()
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

/// NAPI class wrapping a weak handle to a QuickJS runtime engine
/// ([`rquickjs::WeakRuntime`](rquickjs::WeakRuntime)).
#[napi(js_name = "WeakRuntime")]
pub struct JsWeakEngine {
    inner: QjsWeakRuntime,
}

#[napi]
impl JsWeakEngine {
    /// Attempt to promote this weak handle back to a strong [`JsEngine`].
    /// Returns `null` if the engine has already been freed.
    #[napi(js_name = "try_ref")]
    pub fn try_ref(&self, env: Env) -> Result<Option<JsEngine>> {
        if let Some(inner) = self.inner.try_ref() {
            let disposed = Arc::new(AtomicBool::new(false));
            let hook_data = Arc::clone(&disposed);
            env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
                flag.store(true, Ordering::SeqCst);
            })?;
            Ok(Some(JsEngine {
                env,
                disposed,
                inner,
            }))
        } else {
            Ok(None)
        }
    }
}
