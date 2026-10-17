use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::Env;
use napi_derive::napi;
use rquickjs::{runtime::WeakRuntime as QjsWeakRuntime, Runtime as QjsRuntime};
use serde_json::json;

use crate::context::{Context, QjsContext};
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
/// const ctx = new Context();
/// const int = new Atom.from_i64(3);
/// ```
#[napi]
pub struct JsEngine {
    /// The embedded QuickJS engine.
    inner: QjsRuntime,
    /// The host runtime environment, captured at construction.
    env: Env,
    /// Set to `true` when the owning addon (and thus this context) is unloaded.
    alive: Arc<AtomicBool>,
}

#[napi]
impl JsEngine {
    pub(crate) fn runtime(&self) -> &QjsRuntime {
        &self.inner
    }

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

    /// Create a new [`Context`] (boa-backed) and return it.
    ///
    /// Each context owns its own realm and global object, isolated from every
    /// other context. Prefer this over `Context.full()` when you want to manage
    /// contexts through the engine handle.
    #[napi]
    pub fn create_context(&self) -> napi::Result<Context> {
        Context::new()
    }

    /// Create a new QuickJS-backed [`QjsContext`] for use with `Atom`.
    ///
    /// `Atom` is intrinsically QuickJS, so it cannot use the boa-backed
    /// [`Context`]; obtain a `QjsContext` here and pass it to `Atom::from*`.
    #[napi]
    pub fn create_quickjs_context(&self) -> napi::Result<QjsContext> {
        QjsContext::full(self.env)
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

    /// Set a limit (in bytes) on the max stack size the engine will use.
    /// The QuickJS default is `256 * 1024`.
    #[napi]
    pub fn set_max_stack_size(&self, limit: f64) -> napi::Result<()> {
        self.inner.set_max_stack_size(limit as usize);
        Ok(())
    }

    /// Set the memory threshold (in bytes) that triggers garbage collection.
    #[napi]
    pub fn set_gc_threshold(&self, threshold: f64) -> napi::Result<()> {
        self.inner.set_gc_threshold(threshold as usize);
        Ok(())
    }

    /// Set the engine's informational string (visible in debugging tooling).
    #[napi]
    pub fn set_info(&self, info: String) -> napi::Result<()> {
        self.inner.set_info(info).map_err(to_napi_err)
    }

    /// Manually run garbage collection. Most QuickJS values are reference
    /// counted and free themselves automatically; the GC only collects cyclic
    /// references.
    #[napi]
    pub fn run_gc(&self) -> napi::Result<()> {
        self.inner.run_gc();
        Ok(())
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

    /// Whether at least one job (e.g. resolved promise callback) is pending.
    #[napi]
    pub fn is_job_pending(&self) -> napi::Result<bool> {
        Ok(self.inner.is_job_pending())
    }

    /// Execute the first pending job. Returns `true` if a job ran, `false` if the
    /// queue was empty. Throws (as a JS exception carrying the job's error) if a
    /// job threw.
    #[napi]
    pub fn execute_pending_job(&self) -> napi::Result<bool> {
        self.inner.execute_pending_job().map_err(|job_ex| {
            // `JobException` is a thin wrapper around the `Context` the job ran
            // in, so we can read the thrown value out of it.
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
}

/// NAPI class wrapping a weak handle to a QuickJS engine
/// ([`rquickjs::WeakRuntime`](rquickjs::WeakRuntime)).
///
/// Unlike [`JsEngine`], holding a `WeakEngine` does **not** keep the engine
/// alive — use [`JsWeakEngine::try_ref`] to try to re-acquire a strong handle.
/// Once the engine and every context created from it are dropped, `try_ref`
/// returns `null`.
#[napi]
pub struct JsWeakEngine {
    inner: QjsWeakRuntime,
    /// A copy of the host environment and the alive flag, so a promoted handle
    /// can rebuild a full [`JsEngine`].
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

// ---------------------------------------------------------------------------
// Host (N-API) environment surface.
//
// Besides the embedded QuickJS engine above, `Engine` also wraps the *host*
// JavaScript environment (Node's V8) through the `napi::Env` captured at
// construction. These methods let JS build and inspect host values and persist
// host-level instance data. Because the `Env` is stored on the struct, values
// returned here can borrow it for the lifetime of the `Engine` (and so can be
// handed back to JS), and we can detect addon unload via `is_disposed`.
// ---------------------------------------------------------------------------

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
