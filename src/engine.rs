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

/// NAPI class wrapping BOTH:
/// - the embedded QuickJS engine ([`rquickjs::Runtime`]), and
/// - the host N-API environment (`napi::Env`), captured at construction.
///
/// The host `Env` is stored (not injected per method) so host methods can return
/// values that borrow it. A cleanup hook marks the engine disposed when the
/// addon is unloaded.
///
/// JS:
/// ```ts
/// const engine = new Engine();          // or: new Engine(loader)
/// engine.set_memory_limit(1 << 20);
/// const ctx = engine.create_context();
/// const arr = engine.create_array(3);   // host N-API value
/// ```
#[napi(js_name = "Engine")]
#[derive(Clone)]
pub struct JsEngine {
    /// The embedded QuickJS engine.
    pub(crate) inner: QjsRuntime,
    /// The host N-API environment, captured at construction.
    pub(crate) env: Env,
    /// Set to `true` when the owning addon (and thus this context) is unloaded.
    pub(crate) alive: Arc<AtomicBool>,
}

impl JsEngine {
    pub(crate) fn inner(&self) -> &QjsRuntime {
        &self.inner
    }
}

#[napi]
impl JsEngine {
    /// Capture the `napi::Env` that NAPI injects, register a cleanup hook so we
    /// learn when it stops being valid, and create the embedded QuickJS engine.
    ///
    /// Optionally initiate the engine with a module [`crate::loader::JsLoader`],
    /// so `import` statements in this engine's contexts resolve against it. This
    /// is equivalent to constructing bare and then calling
    /// [`JsEngine::set_loader`].
    #[napi(constructor)]
    pub fn new(env: Env, loader: Option<&JsLoader>) -> napi::Result<Self> {
        let alive = Arc::new(AtomicBool::new(false));
        let hook_data = Arc::clone(&alive);
        env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
            flag.store(true, Ordering::SeqCst);
        })?;

        let inner = QjsRuntime::new().map_err(to_napi_err)?;
        if let Some(loader) = loader {
            inner.set_loader(loader.resolver(), loader.loader());
        }
        Ok(JsEngine { inner, env, alive })
    }

    /// Attach (or replace) the module loader/resolver for this engine.
    ///
    /// After this, `import` statements in any of this engine's contexts resolve
    /// through the given `Loader`.
    #[napi]
    pub fn set_loader(&self, loader: &JsLoader) -> napi::Result<()> {
        self.inner.set_loader(loader.resolver(), loader.loader());
        Ok(())
    }

    #[napi(js_name = "set_loader")]
    pub fn set_loader_alias(&self, loader: &JsLoader) -> napi::Result<()> {
        self.set_loader(loader)
    }

    /// Create a `Context` that shares *this* engine.
    ///
    /// Prefer this over a standalone `Context` (i.e. `new Context()`) when you
    /// need several realms on one engine, or when you want the engine configured
    /// (memory limits, etc.) before any context exists. Objects may be shared
    /// between contexts minted from the same engine.
    #[napi]
    pub fn create_context(&self) -> napi::Result<JsContext> {
        let inner = QjsContext::full(&self.inner).map_err(to_napi_err)?;
        Ok(JsContext::from_inner_with_engine(inner, self.clone()))
    }

    #[napi(js_name = "create_context")]
    pub fn create_context_alias(&self) -> napi::Result<JsContext> {
        self.create_context()
    }

    /// Get a weak handle to this engine. Holding it does not keep the engine
    /// alive; [`JsWeakEngine::try_ref`] returns `null` once the engine (and
    /// all contexts created from it) are gone.
    #[napi]
    pub fn weak(&self) -> JsWeakEngine {
        JsWeakEngine {
            inner: self.inner.weak(),
            env: self.env,
            alive: Arc::clone(&self.alive),
        }
    }

    /// Set a limit (in bytes) on the max amount of memory the engine will use.
    /// `0` means unlimited.
    ///
    /// Note: a no-op when a custom allocator is in use (the `rust-alloc` /
    /// `allocator` features — not enabled here).
    #[napi]
    pub fn set_memory_limit(&self, limit: f64) -> napi::Result<()> {
        self.inner.set_memory_limit(limit as usize);
        Ok(())
    }

    #[napi(js_name = "set_memory_limit")]
    pub fn set_memory_limit_alias(&self, limit: f64) -> napi::Result<()> {
        self.set_memory_limit(limit)
    }

    /// Set a limit (in bytes) on the max stack size the engine will use.
    /// The QuickJS default is `256 * 1024`.
    #[napi]
    pub fn set_max_stack_size(&self, limit: f64) -> napi::Result<()> {
        self.inner.set_max_stack_size(limit as usize);
        Ok(())
    }

    #[napi(js_name = "set_max_stack_size")]
    pub fn set_max_stack_size_alias(&self, limit: f64) -> napi::Result<()> {
        self.set_max_stack_size(limit)
    }

    /// Set the memory threshold (in bytes) that triggers garbage collection.
    #[napi]
    pub fn set_gc_threshold(&self, threshold: f64) -> napi::Result<()> {
        self.inner.set_gc_threshold(threshold as usize);
        Ok(())
    }

    #[napi(js_name = "set_gc_threshold")]
    pub fn set_gc_threshold_alias(&self, threshold: f64) -> napi::Result<()> {
        self.set_gc_threshold(threshold)
    }

    /// Set the engine's informational string (visible in debugging tooling).
    #[napi]
    pub fn set_info(&self, info: String) -> napi::Result<()> {
        self.inner.set_info(info).map_err(to_napi_err)
    }

    #[napi(js_name = "set_info")]
    pub fn set_info_alias(&self, info: String) -> napi::Result<()> {
        self.set_info(info)
    }

    /// Manually run garbage collection. Most QuickJS values are reference
    /// counted and free themselves automatically; the GC only collects cyclic
    /// references.
    #[napi]
    pub fn run_gc(&self) -> napi::Result<()> {
        self.inner.run_gc();
        Ok(())
    }

    #[napi(js_name = "run_gc")]
    pub fn run_gc_alias(&self) -> napi::Result<()> {
        self.run_gc()
    }

    /// Memory-usage statistics as a plain object (field names match QuickJS'
    /// `JSMemoryUsage`).
    #[napi]
    pub fn memory_usage(&self) -> napi::Result<serde_json::Value> {
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

    #[napi(js_name = "memory_usage")]
    pub fn memory_usage_alias(&self) -> napi::Result<serde_json::Value> {
        self.memory_usage()
    }

    /// Whether at least one job (e.g. resolved promise callback) is pending.
    #[napi]
    pub fn is_job_pending(&self) -> napi::Result<bool> {
        Ok(self.inner.is_job_pending())
    }

    #[napi(js_name = "is_job_pending")]
    pub fn is_job_pending_alias(&self) -> napi::Result<bool> {
        self.is_job_pending()
    }

    /// Execute the first pending job. Returns `true` if a job ran, `false` if the
    /// queue was empty. Throws (as a JS exception carrying the job's error) if a
    /// job threw.
    #[napi]
    pub fn execute_pending_job(&self) -> napi::Result<bool> {
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

    #[napi(js_name = "execute_pending_job")]
    pub fn execute_pending_job_alias(&self) -> napi::Result<bool> {
        self.execute_pending_job()
    }
}

/// NAPI class wrapping a weak handle to a QuickJS engine
/// ([`rquickjs::WeakRuntime`](rquickjs::WeakRuntime)).
#[napi]
pub struct JsWeakEngine {
    inner: QjsWeakRuntime,
    env: Env,
    alive: Arc<AtomicBool>,
}

#[napi]
impl JsWeakEngine {
    /// Attempt to promote this weak handle back to a strong [`JsEngine`].
    /// Returns `null` if the engine has already been freed.
    #[napi]
    pub fn try_ref(&self) -> napi::Result<Option<JsEngine>> {
        Ok(self.inner.try_ref().map(|inner| JsEngine {
            inner,
            env: self.env,
            alive: Arc::clone(&self.alive),
        }))
    }
}

#[napi]
impl JsEngine {
    /// Whether the owning addon (and thus this context) has been unloaded.
    #[napi]
    pub fn is_disposed(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }

    /// `Engine::create_array` — build a JS array of the requested length.
    #[napi]
    pub fn create_array(&self, len: u32) -> napi::Result<Array<'_>> {
        self.ensure_alive()?;
        let mut raw_val = std::ptr::null_mut();
        napi::check_status!(
            unsafe {
                napi::sys::napi_create_array_with_length(self.env.raw(), len as usize, &mut raw_val)
            },
            "Create array with length failed"
        )?;
        unsafe { Array::from_napi_value(self.env.raw(), raw_val) }
    }

    /// `Engine::create_string` — build a JS string.
    #[napi]
    pub fn create_string(&self, value: String) -> napi::Result<napi::JsString<'_>> {
        self.ensure_alive()?;
        self.env.create_string(&value)
    }

    /// `Engine::create_int32` — build a JS number from an `i32`.
    #[napi]
    pub fn create_int32(&self, int: i32) -> napi::Result<napi::JsNumber<'_>> {
        self.ensure_alive()?;
        self.env.create_int32(int)
    }

    /// `Engine::create_double` — build a JS number from an `f64`.
    #[napi]
    pub fn create_double(&self, value: f64) -> napi::Result<napi::JsNumber<'_>> {
        self.ensure_alive()?;
        self.env.create_double(value)
    }

    /// Does the host global object have this property?
    #[napi]
    pub fn has_global(&self, name: String) -> napi::Result<bool> {
        self.ensure_alive()?;
        let global = self.env.get_global()?;
        global.has_named_property(&name)
    }

    /// `Engine::get_napi_version` — the N-API version of the host runtime.
    #[napi]
    pub fn get_napi_version(&self) -> napi::Result<u32> {
        self.ensure_alive()?;
        self.env.get_napi_version()
    }

    /// `Object::new` through the host context.
    #[napi]
    pub fn make_object(&self) -> napi::Result<Object<'_>> {
        self.ensure_alive()?;
        let mut obj = Object::new(&self.env)?;
        obj.set_named_property("ok", true)?;
        obj.set_named_property("engine", "napi")?;
        Ok(obj)
    }

    /// `Engine::set_instance_data` — persist host-level state on the Agent.
    #[napi]
    pub fn set_instance_data(&self, value: String) -> napi::Result<()> {
        self.ensure_alive()?;
        self.env.set_instance_data(value, (), |_ctx| {})
    }

    /// `Engine::get_instance_data` — read back the persisted host-level state.
    #[napi]
    pub fn get_instance_data(&self) -> napi::Result<Option<String>> {
        self.ensure_alive()?;
        Ok(self
            .env
            .get_instance_data::<String>()?
            .map(|s: &mut String| s.clone()))
    }

    /// Guard: refuse to use an environment that belongs to an unloaded addon.
    fn ensure_alive(&self) -> napi::Result<()> {
        if self.alive.load(Ordering::SeqCst) {
            return Err(Error::new(
                Status::GenericFailure,
                "Engine: N-API context has been disposed (addon unloaded)",
            ));
        }
        Ok(())
    }
}
