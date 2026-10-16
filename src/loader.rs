//! NAPI bindings for QuickJS Loader / Resolver.

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// NAPI JS wrapper around QuickJS Loader configuration.
#[napi]
pub struct Loader {
  pub(crate) base_path: Option<String>,
}

#[napi]
impl Loader {
  /// Create a new module loader with optional base directory path.
  #[napi(constructor)]
  pub fn new(base_path: Option<String>) -> Self {
    Loader { base_path }
  }

  /// Get the configured base path for resolving modules.
  #[napi]
  pub fn get_base_path(&self) -> Option<String> {
    self.base_path.clone()
  }
}
