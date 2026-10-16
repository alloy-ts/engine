//! NAPI bindings for `rquickjs::Runtime`.

use napi_derive::napi;
use rquickjs::Runtime as QjsRuntime;

use crate::loader::Loader;

/// JS-visible wrapper for `rquickjs::Runtime`.
#[napi(js_name = "Runtime")]
pub struct Runtime {
  pub(crate) inner: QjsRuntime,
}

#[napi]
impl Runtime {
  /// Creates a new QuickJS runtime instance, optionally configured with a module Loader.
  #[napi(constructor)]
  pub fn new(loader: Option<&Loader>) -> napi::Result<Self> {
    let inner = QjsRuntime::new().map_err(|e| napi::Error::from_reason(e.to_string()))?;
    if let Some(loader) = loader {
      inner.set_loader(loader.resolver.clone(), loader.loader.clone());
    }
    Ok(Self { inner })
  }

  /// Set the module resolver and loader on this runtime.
  #[napi]
  pub fn set_loader(&self, loader: &Loader) {
    self.inner.set_loader(loader.resolver.clone(), loader.loader.clone());
  }

  /// Run garbage collection cycle on the QuickJS runtime.
  #[napi]
  pub fn run_gc(&self) {
    self.inner.run_gc();
  }

  /// Set memory limit for the QuickJS runtime in bytes.
  #[napi]
  pub fn set_memory_limit(&self, limit: u32) {
    self.inner.set_memory_limit(limit as usize);
  }

  /// Set max stack size for the QuickJS runtime in bytes.
  #[napi]
  pub fn set_max_stack_size(&self, limit: u32) {
    self.inner.set_max_stack_size(limit as usize);
  }
}
