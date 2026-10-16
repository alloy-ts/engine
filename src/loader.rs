//! NAPI bindings for QuickJS module loader and resolver.

use napi_derive::napi;
use rquickjs::loader::{BuiltinLoader, BuiltinResolver, FileResolver, ScriptLoader};

/// Resolver that resolves module names against file system paths.
#[napi]
pub struct NativeFileResolver {
  pub(crate) inner: Option<FileResolver>,
}

#[napi]
impl NativeFileResolver {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: Some(FileResolver::default()),
    }
  }

  #[napi(js_name = "add_path")]
  pub fn add_path(&mut self, path: String) {
    if let Some(ref mut resolver) = self.inner {
      resolver.add_path(path);
    }
  }
}

/// Resolver for built-in modules.
#[napi]
pub struct NativeBuiltinResolver {
  pub(crate) inner: Option<BuiltinResolver>,
}

#[napi]
impl NativeBuiltinResolver {
  #[napi(constructor)]
  pub fn new(names: Vec<String>) -> Self {
    let mut resolver = BuiltinResolver::default();
    for name in names {
      resolver.add_module(name);
    }
    Self {
      inner: Some(resolver),
    }
  }
}

/// Loader for raw JavaScript source scripts.
#[napi]
pub struct NativeScriptLoader {
  pub(crate) inner: Option<ScriptLoader>,
}

#[napi]
impl NativeScriptLoader {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: Some(ScriptLoader::default()),
    }
  }
}

/// Loader for built-in modules.
#[napi]
pub struct NativeBuiltinLoader {
  #[allow(dead_code)]
  pub(crate) inner: Option<BuiltinLoader>,
}

#[napi]
impl NativeBuiltinLoader {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: Some(BuiltinLoader::default()),
    }
  }
}
