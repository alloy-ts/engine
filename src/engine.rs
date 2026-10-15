use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::sys;
use napi_derive::napi;

/// JS class wrapping the N-API context (`Env`).
#[napi]
pub struct Engine(Env, Arc<AtomicBool>);

#[napi]
impl Engine {
    /// Capture the `Env` that the runtime injects and register a
    /// cleanup hook so we learn when it stops being valid.
    #[napi(constructor)]
    pub fn new(env: Env) -> Result<Self> {
        let disposed = Arc::new(AtomicBool::new(false));

        let hook_data = Arc::clone(&disposed);
        env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
            flag.store(true, Ordering::SeqCst);
        })?;

        Ok(Self(env, disposed))
    }

    /// Whether the owning addon (and thus this context) has been unloaded.
    #[napi(js_name = "is_disposed")]
    pub fn is_disposed(&self) -> bool {
        self.1.load(Ordering::SeqCst)
    }

    #[napi(js_name = "create_array")]
    pub fn create_array(&self, len: u32) -> Result<Array<'_>> {
        self.ensure_alive()?;
        self.0.create_array(len)
    }

    /// `Engine::create_string` — build a JS string.
    #[napi(js_name = "create_string")]
    pub fn create_string(&self, value: String) -> Result<Unknown<'_>> {
        self.ensure_alive()?;
        self.0.create_string(&value)?.into_unknown(&self.0)
    }

    /// `Engine::create_int32` — build a JS number from an `i32`.
    #[napi(js_name = "create_int32")]
    pub fn create_int32(&self, int: i32) -> Result<Unknown<'_>> {
        self.ensure_alive()?;
        self.0.create_int32(int)?.into_unknown(&self.0)
    }

    /// `Engine::create_double` — build a JS number from an `f64`.
    #[napi(js_name = "create_double")]
    pub fn create_double(&self, value: f64) -> Result<Unknown<'_>> {
        self.ensure_alive()?;
        self.0.create_double(value)?.into_unknown(&self.0)
    }

    /// Does the global object have this property?
    #[napi(js_name = "has_global")]
    pub fn has_global(&self, name: String) -> Result<bool> {
        self.ensure_alive()?;
        self.0.get_global()?.has_named_property(&name)
    }

    /// `Engine::get_napi_version` — the N-API version of the host runtime.
    #[napi(js_name = "get_napi_version")]
    pub fn get_napi_version(&self) -> Result<u32> {
        self.ensure_alive()?;
        self.0.get_napi_version()
    }

    /// `Object::new` through the captured context.
    #[napi(js_name = "make_object")]
    pub fn make_object(&self) -> Result<Object<'_>> {
        self.ensure_alive()?;
        let mut obj = Object::new(&self.0)?;
        obj.set_named_property("ok", true)?;
        obj.set_named_property("engine", "napi")?;
        Ok(obj)
    }

    /// `Engine::set_instance_data` — persist VM-specific state on the Agent.
    #[napi(js_name = "set_instance_data")]
    pub fn set_instance_data(&self, value: String) -> Result<()> {
        self.ensure_alive()?;
        let data_ptr = Box::into_raw(Box::new(value));
        unsafe extern "C" fn finalize_string(_env: sys::napi_env, data: *mut std::ffi::c_void, _hint: *mut std::ffi::c_void) {
            if !data.is_null() {
                drop(Box::from_raw(data as *mut String));
            }
        }
        let status = unsafe {
            sys::napi_set_instance_data(
                self.0.raw(),
                data_ptr as *mut _,
                Some(finalize_string),
                std::ptr::null_mut(),
            )
        };
        napi::check_status!(status, "Failed to set instance data")?;
        Ok(())
    }

    /// `Engine::get_instance_data` — read back the persisted VM-specific state.
    #[napi(js_name = "get_instance_data")]
    pub fn get_instance_data(&self) -> Result<Option<String>> {
        self.ensure_alive()?;
        let mut data_ptr = std::ptr::null_mut();
        let status = unsafe { sys::napi_get_instance_data(self.0.raw(), &mut data_ptr) };
        napi::check_status!(status, "Failed to get instance data")?;
        if data_ptr.is_null() {
            Ok(None)
        } else {
            let s = unsafe { &*(data_ptr as *const String) };
            Ok(Some(s.clone()))
        }
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
