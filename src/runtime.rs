use napi::Error;
use napi_derive::napi;
use rquickjs::{
    runtime::WeakRuntime as QjsWeakRuntime, Context as QjsContext, Runtime as QjsRuntime,
};
use serde_json::json;

use crate::context::JsContext;

/// Map a [`rquickjs::Error`] into a `napi::Error` so JS sees a thrown exception.
fn to_napi_err(e: rquickjs::Error) -> Error {
    Error::from_reason(e.to_string())
}
#[napi(js_name = "Runtime")]
pub struct JsRuntime {
    /// The embedded QuickJS runtime engine.
    inner: QjsRuntime,
}

#[napi]
impl JsRuntime {
    /// Create a new runtime. Generally only fails if memory was unavailable.
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        let inner = QjsRuntime::new().map_err(to_napi_err)?;
        Ok(JsRuntime { inner })
    }

    /// Configure module loading and resolution using a [`JsLoader`].
    #[napi(js_name = "set_loader")]
    pub fn set_loader(&self, loader: &crate::loader::JsLoader) -> napi::Result<()> {
        crate::loader::apply_loader_to_runtime(&self.inner, loader);
        Ok(())
    }

    /// Create a `Context` that shares *this* runtime's engine.
    ///
    /// Prefer this over a standalone `Context` (i.e. `new Context()`) when you
    /// need several realms on one engine, or when you want the engine configured
    /// (memory limits, etc.) before any context exists. Objects may be shared
    /// between contexts minted from the same runtime.
    #[napi(js_name = "create_context")]
    pub fn create_context(&self) -> napi::Result<JsContext> {
        let inner = QjsContext::full(&self.inner).map_err(to_napi_err)?;
        Ok(JsContext::from_inner(inner))
    }

    /// Get a weak handle to this runtime. Holding it does not keep the runtime
    /// alive; [`JsWeakRuntime::try_ref`] returns `null` once the runtime (and
    /// all contexts created from it) are gone.
    #[napi]
    pub fn weak(&self) -> JsWeakRuntime {
        JsWeakRuntime {
            inner: self.inner.weak(),
        }
    }

    /// Set a limit (in bytes) on the max amount of memory the runtime will use.
    /// `0` means unlimited.
    ///
    /// Note: a no-op when a custom allocator is in use (the `rust-alloc` /
    /// `allocator` features — not enabled here).
    #[napi(js_name = "set_memory_limit")]
    pub fn set_memory_limit(&self, limit: f64) -> napi::Result<()> {
        self.inner.set_memory_limit(limit as usize);
        Ok(())
    }

    /// Set a limit (in bytes) on the max stack size the runtime will use.
    /// The QuickJS default is `256 * 1024`.
    #[napi(js_name = "set_max_stack_size")]
    pub fn set_max_stack_size(&self, limit: f64) -> napi::Result<()> {
        self.inner.set_max_stack_size(limit as usize);
        Ok(())
    }

    /// Set the memory threshold (in bytes) that triggers garbage collection.
    #[napi(js_name = "set_gc_threshold")]
    pub fn set_gc_threshold(&self, threshold: f64) -> napi::Result<()> {
        self.inner.set_gc_threshold(threshold as usize);
        Ok(())
    }

    /// Set the runtime's informational string (visible in debugging tooling).
    #[napi(js_name = "set_info")]
    pub fn set_info(&self, info: String) -> napi::Result<()> {
        self.inner.set_info(info).map_err(to_napi_err)
    }

    /// Manually run garbage collection. Most QuickJS values are reference
    /// counted and free themselves automatically; the GC only collects cyclic
    /// references.
    #[napi(js_name = "run_gc")]
    pub fn run_gc(&self) -> napi::Result<()> {
        self.inner.run_gc();
        Ok(())
    }

    /// Memory-usage statistics as a plain object (field names match QuickJS'
    /// `JSMemoryUsage`).
    #[napi(js_name = "memory_usage")]
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
    #[napi(js_name = "is_job_pending")]
    pub fn is_job_pending(&self) -> napi::Result<bool> {
        Ok(self.inner.is_job_pending())
    }

    /// Execute the first pending job. Returns `true` if a job ran, `false` if the
    /// queue was empty. Throws (as a JS exception carrying the job's error) if a
    /// job threw.
    #[napi(js_name = "execute_pending_job")]
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

/// NAPI class wrapping a weak handle to a QuickJS runtime
/// ([`rquickjs::WeakRuntime`](rquickjs::WeakRuntime)).
///
/// Unlike [`JsRuntime`], holding a `WeakRuntime` does **not** keep the engine
/// alive — use [`JsWeakRuntime::try_ref`] to try to re-acquire a strong handle.
/// Once the runtime and every context created from it are dropped, `try_ref`
/// returns `null`.
#[napi(js_name = "WeakRuntime")]
pub struct JsWeakRuntime {
    inner: QjsWeakRuntime,
}

#[napi]
impl JsWeakRuntime {
    /// Attempt to promote this weak handle back to a strong [`JsRuntime`].
    /// Returns `null` if the runtime has already been freed.
    #[napi(js_name = "try_ref")]
    pub fn try_ref(&self) -> napi::Result<Option<JsRuntime>> {
        Ok(self.inner.try_ref().map(|inner| JsRuntime { inner }))
    }
}
