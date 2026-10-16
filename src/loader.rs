//! NAPI bindings for QuickJS loader and resolver.

use napi_derive::napi;

/// JS wrapper for a module loader & resolver pair.
#[napi(js_name = "Loader")]
pub struct Loader;

#[napi]
impl Loader {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self
    }
}

impl Default for Loader {
    fn default() -> Self {
        Self::new()
    }
}
