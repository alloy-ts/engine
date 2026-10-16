//! NAPI bindings for module loading and resolving in rquickjs.

use napi_derive::napi;

/// A JS-configurable module loader/resolver.
#[napi]
pub struct QuickLoader {
  // Builtin/File module loader placeholder configuration
}

#[napi]
impl QuickLoader {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {}
  }
}
