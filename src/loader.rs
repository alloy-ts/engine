//! NAPI bindings for `rquickjs::loader`.

use std::sync::{Arc, Mutex};
use napi_derive::napi;
use rquickjs::loader::{BuiltinLoader, BuiltinResolver, ImportAttributes, Loader as QjsLoader, Resolver as QjsResolver};
use rquickjs::{Ctx, Module, Result as QjsResult};

#[derive(Clone)]
pub struct NapiResolver(pub Arc<Mutex<BuiltinResolver>>);

impl QjsResolver for NapiResolver {
  fn resolve<'js>(
    &mut self,
    ctx: &Ctx<'js>,
    base: &str,
    name: &str,
    attributes: Option<ImportAttributes<'js>>,
  ) -> QjsResult<String> {
    self.0.lock().unwrap().resolve(ctx, base, name, attributes)
  }
}

#[derive(Clone)]
pub struct NapiLoader(pub Arc<Mutex<BuiltinLoader>>);

impl QjsLoader for NapiLoader {
  fn load<'js>(
    &mut self,
    ctx: &Ctx<'js>,
    name: &str,
    attributes: Option<ImportAttributes<'js>>,
  ) -> QjsResult<Module<'js>> {
    self.0.lock().unwrap().load(ctx, name, attributes)
  }
}

/// JS-visible wrapper for module loader options and builtin module registration.
#[napi(js_name = "Loader")]
pub struct Loader {
  pub(crate) resolver: NapiResolver,
  pub(crate) loader: NapiLoader,
}

#[napi]
impl Loader {
  /// Create a new module loader.
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      resolver: NapiResolver(Arc::new(Mutex::new(BuiltinResolver::default()))),
      loader: NapiLoader(Arc::new(Mutex::new(BuiltinLoader::default()))),
    }
  }

  /// Add a builtin module to the resolver and loader.
  #[napi]
  pub fn add_module(&mut self, name: String, source: String) {
    if let Ok(mut resolver) = self.resolver.0.lock() {
      resolver.add_module(name.clone());
    }
    if let Ok(mut loader) = self.loader.0.lock() {
      loader.add_module(name, source);
    }
  }
}

impl Default for Loader {
  fn default() -> Self {
    Self::new()
  }
}
