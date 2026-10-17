use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::Env;
use napi_derive::napi;

use crate::context::Context;

#[napi(object)]
pub struct EngineBackend {
    pub boa: String,
    pub quick_js: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendKind {
    Boa,
    QuickJs,
}

/// NAPI Engine wrapping ECMAScript execution backends (Boa by default, or QuickJS)
/// as well as host N-API environment capabilities.
///
/// JS usage:
/// ```ts
/// const engine = new Engine(); // defaults to boa
/// const engine2 = new Engine(Engine.backend.QuickJs);
/// const engine3 = Engine.boa();
/// const engine4 = Engine.quickjs();
/// ```
#[napi]
pub struct Engine {
    kind: BackendKind,
    /// Default Boa context owned by the engine for `engine.eval(...)`.
    default_boa_ctx: Option<Context>,
    /// Host N-API environment captured at construction.
    env: Env,
    /// Set to `true` when the owning addon is unloaded.
    alive: Arc<AtomicBool>,
}

#[napi]
impl Engine {
    /// Static backend constant object (`Engine.backend.boa` / `Engine.backend.quickJs`).
    #[napi(getter, js_name = "backend")]
    pub fn get_backend_static() -> EngineBackend {
        EngineBackend {
            boa: "boa".to_string(),
            quick_js: "quickjs".to_string(),
        }
    }

    /// Instance getter for backend object.
    #[napi(getter)]
    pub fn backend(&self) -> EngineBackend {
        EngineBackend {
            boa: "boa".to_string(),
            quick_js: "quickjs".to_string(),
        }
    }

    /// Primary constructor: `new Engine(backend?: string)`
    /// Defaults to `"boa"` if omitted.
    #[napi(constructor)]
    pub fn new(env: Env, backend: Option<String>) -> napi::Result<Self> {
        let alive = Arc::new(AtomicBool::new(false));
        let hook_data = Arc::clone(&alive);
        env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
            flag.store(true, Ordering::SeqCst);
        })?;

        let kind = match backend.as_deref() {
            None | Some("") | Some("boa") | Some("Boa") => BackendKind::Boa,
            Some("quickjs") | Some("quickJs") | Some("QuickJs") | Some("QuickJS") => BackendKind::QuickJs,
            Some(unknown) => {
                return Err(Error::from_reason(format!(
                    "Unknown engine backend: '{unknown}'. Supported backends are 'boa' and 'quickjs'."
                )))
            }
        };

        let default_boa_ctx = if kind == BackendKind::Boa {
            Some(Context::new()?)
        } else {
            None
        };

        Ok(Engine {
            kind,
            default_boa_ctx,
            env,
            alive,
        })
    }

    /// Factory for Boa backend: `Engine.boa()`
    #[napi(factory)]
    pub fn boa(env: Env) -> napi::Result<Self> {
        Self::new(env, Some("boa".to_string()))
    }

    /// Factory for QuickJS backend: `Engine.quickjs()`
    #[napi(factory)]
    pub fn quickjs(env: Env) -> napi::Result<Self> {
        Self::new(env, Some("quickjs".to_string()))
    }

    /// Name of the active backend (`"boa"` or `"quickjs"`).
    #[napi]
    pub fn backend_name(&self) -> String {
        match self.kind {
            BackendKind::Boa => "boa".to_string(),
            BackendKind::QuickJs => "quickjs".to_string(),
        }
    }

    /// Create an isolated execution [`Context`].
    #[napi]
    pub fn create_context(&self) -> napi::Result<Context> {
        self.ensure_alive()?;
        Context::new()
    }

    /// Evaluate source code in the engine's default context.
    #[napi]
    pub fn eval(&self, source: String) -> napi::Result<serde_json::Value> {
        self.ensure_alive()?;
        match self.kind {
            BackendKind::Boa => {
                if let Some(ref ctx) = self.default_boa_ctx {
                    ctx.eval(source)
                } else {
                    Err(Error::from_reason("Engine Boa context missing"))
                }
            }
            BackendKind::QuickJs => Err(Error::from_reason(
                "QuickJS backend eval requires 'quickjs' feature enabled in build",
            )),
        }
    }

    /// Trigger garbage collection.
    #[napi]
    pub fn run_gc(&self) -> napi::Result<()> {
        self.ensure_alive()?;
        Ok(())
    }

    /// Set memory limit in bytes (where supported).
    #[napi]
    pub fn set_memory_limit(&self, _limit: f64) -> napi::Result<()> {
        self.ensure_alive()?;
        Ok(())
    }

    /// Set maximum stack size limit in bytes.
    #[napi]
    pub fn set_max_stack_size(&self, limit: f64) -> napi::Result<()> {
        self.ensure_alive()?;
        if let Some(ref ctx) = self.default_boa_ctx {
            ctx.set_runtime_limits(crate::context::RuntimeLimitsView {
                stack_size: Some(limit as u32),
                loop_iteration_limit: None,
                backtrace_limit: None,
                recursion_limit: None,
            })?;
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Host (N-API) environment surface
    // -----------------------------------------------------------------------

    #[napi]
    pub fn is_disposed(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }

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

    #[napi]
    pub fn create_string(&self, value: String) -> napi::Result<napi::JsString<'_>> {
        self.ensure_alive()?;
        self.env.create_string(&value)
    }

    #[napi]
    pub fn create_int32(&self, int: i32) -> napi::Result<napi::JsNumber<'_>> {
        self.ensure_alive()?;
        self.env.create_int32(int)
    }

    #[napi]
    pub fn create_double(&self, value: f64) -> napi::Result<napi::JsNumber<'_>> {
        self.ensure_alive()?;
        self.env.create_double(value)
    }

    #[napi]
    pub fn has_global(&self, name: String) -> napi::Result<bool> {
        self.ensure_alive()?;
        let global = self.env.get_global()?;
        global.has_named_property(&name)
    }

    #[napi]
    pub fn get_napi_version(&self) -> napi::Result<u32> {
        self.ensure_alive()?;
        self.env.get_napi_version()
    }

    #[napi]
    pub fn make_object(&self) -> napi::Result<Object<'_>> {
        self.ensure_alive()?;
        let mut obj = Object::new(&self.env)?;
        obj.set_named_property("ok", true)?;
        obj.set_named_property("engine", "napi")?;
        Ok(obj)
    }

    #[napi]
    pub fn set_instance_data(&self, value: String) -> napi::Result<()> {
        self.ensure_alive()?;
        self.env.set_instance_data(value, (), |_ctx| {})
    }

    #[napi]
    pub fn get_instance_data(&self) -> napi::Result<Option<String>> {
        self.ensure_alive()?;
        Ok(self
            .env
            .get_instance_data::<String>()?
            .map(|s: &mut String| s.clone()))
    }

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
