//! NAPI bindings for QuickJS module Loader and Resolver.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::loader::{
  BuiltinLoader, BuiltinResolver, FileResolver, NativeLoader, ScriptLoader,
};

/// NAPI binding for module resolver configuration.
#[napi]
pub struct Resolver {
  pub(crate) builtin: Option<BuiltinResolver>,
  pub(crate) file: Option<FileResolver>,
}

#[napi]
impl Resolver {
  /// Create a new Resolver builder.
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      builtin: None,
      file: None,
    }
  }

  /// Enable file-system module resolution relative to base directory or path.
  #[napi]
  pub fn with_file_resolver(&mut self) {
    let mut resolver = FileResolver::default();
    resolver.add_path(".");
    self.file = Some(resolver);
  }

  /// Add a search path to file resolver.
  #[napi]
  pub fn add_path(&mut self, path: String) {
    if let Some(ref mut file_resolver) = self.file {
      file_resolver.add_path(path);
    } else {
      let mut resolver = FileResolver::default();
      resolver.add_path(path);
      self.file = Some(resolver);
    }
  }
}

/// NAPI binding for module loader configuration.
#[napi]
pub struct Loader {
  pub(crate) builtin: Option<BuiltinLoader>,
  pub(crate) script: Option<ScriptLoader>,
  pub(crate) native: Option<NativeLoader>,
}

#[napi]
impl Loader {
  /// Create a new Loader builder.
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      builtin: None,
      script: None,
      native: None,
    }
  }

  /// Enable script module loading (loads JS files from file system).
  #[napi]
  pub fn with_script_loader(&mut self) {
    self.script = Some(ScriptLoader::default());
  }

  /// Enable builtin module loader.
  #[napi]
  pub fn with_builtin_loader(&mut self) {
    self.builtin = Some(BuiltinLoader::default());
  }
}
