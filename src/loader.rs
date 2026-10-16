//! NAPI bindings for module loading support in QuickJS.

use napi_derive::napi;
use rquickjs::loader::{BuiltinLoader, BuiltinResolver, FileResolver, ScriptLoader};

/// Builtin module loader/resolver wrapper for NAPI.
#[napi(js_name = "BuiltinResolver")]
pub struct JsBuiltinResolver {
  pub(crate) inner: Option<BuiltinResolver>,
}

#[napi]
impl JsBuiltinResolver {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: Some(BuiltinResolver::default()),
    }
  }

  #[napi(js_name = "add_module")]
  pub fn add_module(&mut self, name: String) {
    if let Some(ref mut inner) = self.inner {
      inner.add_module(name);
    }
  }
}

/// File resolver wrapper for NAPI.
#[napi(js_name = "FileResolver")]
pub struct JsFileResolver {
  pub(crate) inner: Option<FileResolver>,
}

#[napi]
impl JsFileResolver {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: Some(FileResolver::default()),
    }
  }

  #[napi(js_name = "add_path")]
  pub fn add_path(&mut self, path: String) {
    if let Some(ref mut inner) = self.inner {
      inner.add_path(path);
    }
  }
}

/// Builtin module loader wrapper for NAPI.
#[napi(js_name = "BuiltinLoader")]
pub struct JsBuiltinLoader {
  pub(crate) inner: Option<BuiltinLoader>,
}

#[napi]
impl JsBuiltinLoader {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: Some(BuiltinLoader::default()),
    }
  }
}

/// Script loader wrapper for NAPI.
#[napi(js_name = "ScriptLoader")]
pub struct JsScriptLoader {
  pub(crate) inner: Option<ScriptLoader>,
}

#[napi]
impl JsScriptLoader {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: Some(ScriptLoader::default()),
    }
  }
}
