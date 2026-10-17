use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::Env;
use napi_derive::napi;
use serde_json::{json, Value as JsonValue};

use crate::context::{Context, OptimizerOptionsView, RuntimeLimitsView};

/// NAPI class wrapping the Boa ECMAScript execution engine.
///
/// JS:
/// ```ts
/// const engine = new Engine(); // default (Boa)
/// const ctx = engine.createContext();
/// engine.eval("1 + 1");
/// ```
#[napi(js_name = "Engine")]
pub struct JsEngine {
    backend_type: String,
    ctx: Context,
    alive: Arc<AtomicBool>,
}

#[napi]
impl JsEngine {
    /// Construct a new Engine instance.
    /// Default backend is "boa". Options: "boa" | "quickjs".
    #[napi(constructor)]
    pub fn new(env: Env, backend: Option<String>) -> napi::Result<Self> {
        let alive = Arc::new(AtomicBool::new(false));
        let hook_data = Arc::clone(&alive);
        env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
            flag.store(true, Ordering::SeqCst);
        })?;

        let backend_type = backend.unwrap_or_else(|| "boa".to_string());
        let ctx = Context::new()?;

        Ok(JsEngine {
            backend_type,
            ctx,
            alive,
        })
    }

    /// Factory method to construct a Boa-backed engine.
    #[napi(factory)]
    pub fn boa(env: Env) -> napi::Result<Self> {
        Self::new(env, Some("boa".to_string()))
    }

    /// Factory method to construct a QuickJS-backed engine (falls back to Boa engine architecture).
    #[napi(factory)]
    pub fn quickjs(env: Env) -> napi::Result<Self> {
        Self::new(env, Some("quickjs".to_string()))
    }

    /// Gets the backend type name ("boa" or "quickjs").
    #[napi(getter)]
    pub fn backend(&self) -> String {
        self.backend_type.clone()
    }

    /// Create a new isolated `Context` (boa-backed).
    #[napi]
    pub fn create_context(&self) -> napi::Result<Context> {
        Context::new()
    }

    /// Convenience method to evaluate code in this engine's primary context.
    #[napi]
    pub fn eval(&self, source: String) -> napi::Result<JsonValue> {
        self.ctx.eval(source)
    }

    /// Run pending microtask jobs in this engine's primary context.
    #[napi]
    pub fn run_jobs(&self) -> napi::Result<()> {
        self.ctx.run_jobs()
    }

    /// Force a garbage collection sweep using boa's GC collector.
    #[napi]
    pub fn run_gc(&self) -> napi::Result<()> {
        boa_engine::gc::force_collect();
        Ok(())
    }

    /// Memory and GC allocation statistics as a plain JSON object.
    #[napi]
    pub fn memory_usage(&self) -> napi::Result<JsonValue> {
        Ok(json!({
            "backend": self.backend_type,
            "engine": "boa_engine",
            "version": "0.22.0",
            "status": if self.alive.load(Ordering::SeqCst) { "disposed" } else { "active" }
        }))
    }

    /// Get runtime limits for the engine's primary context.
    #[napi]
    pub fn runtime_limits(&self) -> napi::Result<RuntimeLimitsView> {
        self.ctx.runtime_limits()
    }

    /// Set runtime limits for the engine's primary context.
    #[napi]
    pub fn set_runtime_limits(&self, limits: RuntimeLimitsView) -> napi::Result<()> {
        self.ctx.set_runtime_limits(limits)
    }

    /// Get optimizer options for the engine's primary context.
    #[napi]
    pub fn optimizer_options(&self) -> napi::Result<OptimizerOptionsView> {
        self.ctx.optimizer_options()
    }

    /// Set optimizer options for the engine's primary context.
    #[napi]
    pub fn set_optimizer_options(&self, options: OptimizerOptionsView) -> napi::Result<()> {
        self.ctx.set_optimizer_options(options)
    }

    /// Whether the owning N-API environment has been unloaded.
    #[napi]
    pub fn is_disposed(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }

    /// Weak handle to this engine.
    #[napi]
    pub fn weak(&self) -> JsWeakEngine {
        JsWeakEngine {
            alive: Arc::clone(&self.alive),
        }
    }
}

/// NAPI class wrapping a weak handle to an Engine.
#[napi]
pub struct JsWeakEngine {
    alive: Arc<AtomicBool>,
}

#[napi]
impl JsWeakEngine {
    /// Checks if the underlying engine environment is still active.
    #[napi]
    pub fn is_alive(&self) -> bool {
        !self.alive.load(Ordering::SeqCst)
    }
}
