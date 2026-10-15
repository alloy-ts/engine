//! `Engine`: a JS-visible NAPI class that owns the N-API context handle.
//!
//! The N-API context is `napi::Engine` (aliased below as `NapiEngine`). We
//! cannot put `#[napi]` on `napi::Engine` itself — it is a foreign type — so
//! our `Engine` *wraps* it and exposes a safe subset as JavaScript methods.
//!
//! ## The `napi::Engine` contract
//!
//! 1. **Reuse the same context for nested calls.** Every method does all of its
//!    N-API work through `self.0` — the very handle that entered the call.
//! 2. **Never cache it for general reuse, and never share it between addon
//!    instances or Worker threads.** Each `Engine` captures its *own* handle at
//!    construction time, and the fields are private so it cannot leak out.
//! 3. **It is invalid once the addon instance is unloaded.** We register an
//!    engine cleanup hook (see [`Engine::new`]) that flips the disposed
//!    flag, so every public method refuses to use a dangling handle. VM-specific
//!    state lives in `set_instance_data`, which is notified on the same event.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::{Error, Result, Status};
use napi::{Engine as NapiEngine, JsGlobal, Object, Unknown};

/// JS class wrapping the N-API context (`napi::Engine`).
///
/// JS: `const engine = new Engine();` — the context is injected by NAPI, so the
/// constructor takes no JavaScript arguments.
#[napi]
pub struct Engine(NapiEngine, Arc<AtomicBool>);

#[napi]
impl Engine {
    /// Capture the `napi::Engine` that the runtime injects and register a
    /// cleanup hook so we learn when it stops being valid.
    #[napi(constructor)]
    pub fn new(engine: NapiEngine) -> Result<Self> {
        let disposed = Arc::new(AtomicBool::new(false));

        // Share the flag with the cleanup hook: when the environment (and so the
        // addon instance) is torn down, mark this engine disposed.
        let hook_data = Arc::clone(&disposed);
        engine.add_engine_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
            flag.store(true, Ordering::SeqCst);
        })?;

        Ok(Self(engine, disposed))
    }

    /// Whether the owning addon (and thus this context) has been unloaded.
    #[napi]
    pub fn is_disposed(&self) -> bool {
        self.1.load(Ordering::SeqCst)
    }

    #[napi]
    pub fn create_array(&self, len: u32) -> Result<Unknown> {
        self.ensure_alive()?;
        Ok(self.0.create_array(len)?.into_unknown())
    }

    /// `Engine::create_string` — build a JS string.
    #[napi]
    pub fn create_string(&self, value: String) -> Result<Unknown> {
        self.ensure_alive()?;
        Ok(self.0.create_string(&value)?.into_unknown())
    }

    /// `Engine::create_int32` — build a JS number from an `i32`.
    #[napi]
    pub fn create_int32(&self, int: i32) -> Result<Unknown> {
        self.ensure_alive()?;
        Ok(self.0.create_int32(int)?.into_unknown())
    }

    /// `Engine::create_double` — build a JS number from an `f64`.
    #[napi]
    pub fn create_double(&self, value: f64) -> Result<Unknown> {
        self.ensure_alive()?;
        Ok(self.0.create_double(value)?.into_unknown())
    }

    /// Resolve the JS global object (mirrors `napi::Engine::get_global`).
    pub fn get_global(&self) -> Result<JsGlobal<'static>> {
        self.0.get_global()
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
        self.0.get_napi_version()
    }

    /// `Object::new` through the captured context.
    #[napi]
    pub fn make_object(&self) -> Result<Object> {
        self.ensure_alive()?;
        let mut obj = Object::new(&self.0)?;
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
        self.0.set_instance_data(value, (), |_ctx| {})
    }

    /// `Engine::get_instance_data` — read back the persisted VM-specific state.
    #[napi]
    pub fn get_instance_data(&self) -> Result<Option<String>> {
        self.ensure_alive()?;
        Ok(self.0.get_instance_data::<String>()?.map(|s| s.clone()))
    }

    /// Guard: refuse to use a context that belongs to an unloaded addon.
    fn ensure_alive(&self) -> Result<()> {
        if self.1.load(Ordering::SeqCst) {
            return Err(Error::new(
                Status::GenericFailure,
                "Engine: N-API context has been disposed (addon unloaded)",
            ));
        }
        Ok(())
    }
}
