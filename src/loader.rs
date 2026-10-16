//! NAPI bindings for QuickJS `Loader` and `Resolver`.

use napi_derive::napi;
use rquickjs::loader::{BuiltinLoader, BuiltinResolver, FileResolver, ScriptLoader};

/// Built-in module loader.
#[napi]
pub struct BuiltinModuleLoader {
  pub(crate) _inner: BuiltinLoader,
}

#[napi]
impl BuiltinModuleLoader {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      _inner: BuiltinLoader::default(),
    }
  }
}

/// Built-in module resolver.
#[napi]
pub struct BuiltinModuleResolver {
  pub(crate) _inner: BuiltinResolver,
}

#[napi]
impl BuiltinModuleResolver {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      _inner: BuiltinResolver::default(),
    }
  }
}

/// File resolver for loading modules from the filesystem.
#[napi]
pub struct FileModuleResolver {
  pub(crate) _inner: FileResolver,
}

#[napi]
impl FileModuleResolver {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      _inner: FileResolver::default(),
    }
  }
}

/// Script loader for evaluating script files.
#[napi]
pub struct ScriptModuleLoader {
  pub(crate) _inner: ScriptLoader,
}

#[napi]
impl ScriptModuleLoader {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      _inner: ScriptLoader::default(),
    }
  }
}
