//! NAPI bindings for QuickJS Atom.

use napi_derive::napi;

/// A QuickJS Atom NAPI wrapper.
#[napi(js_name = "Atom")]
pub struct Atom;

#[napi]
impl Atom {
    #[napi(constructor)]
    pub fn new() -> Self {
        Atom
    }
}

impl Default for Atom {
    fn default() -> Self {
        Self::new()
    }
}
