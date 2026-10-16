//! NAPI bindings for QuickJS [`Loader`] and [`Resolver`].

use napi_derive::napi;

use rquickjs::loader::{FileResolver, ScriptLoader};

/// NAPI wrapper for [`FileResolver`].
#[napi(js_name = "FileResolver")]
#[derive(Default)]
pub struct JsFileResolver {
  #[allow(dead_code)]
  pub(crate) inner: FileResolver,
}

#[napi]
impl JsFileResolver {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self::default()
  }

  /// Add a base search path for module resolution.
  #[napi(js_name = "add_path")]
  pub fn add_path(&mut self, path: String) {
    self.inner.add_path(path);
  }
}

/// NAPI wrapper for [`ScriptLoader`].
#[napi(js_name = "ScriptLoader")]
#[derive(Default)]
pub struct JsScriptLoader {
  #[allow(dead_code)]
  pub(crate) inner: ScriptLoader,
}

#[napi]
impl JsScriptLoader {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self::default()
  }
}
