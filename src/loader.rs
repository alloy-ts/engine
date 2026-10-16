//! NAPI bindings for QuickJS [`Loader`] and [`Resolver`].

use napi_derive::napi;

/// NAPI wrapper around a QuickJS module loader.
#[napi]
pub struct Loader;

#[napi]
impl Loader {
  #[napi(constructor)]
  pub fn new() -> Self {
    Loader
  }
}

/// NAPI wrapper around a QuickJS module resolver.
#[napi]
pub struct Resolver;

#[napi]
impl Resolver {
  #[napi(constructor)]
  pub fn new() -> Self {
    Resolver
  }

  /// Resolve a module specifier relative to a base path.
  #[napi]
  pub fn resolve(&self, base: String, name: String) -> String {
    if !name.starts_with('.') {
      name
    } else {
      let mut split = base.rsplitn(2, '/');
      let path = match (split.next(), split.next()) {
        (_, Some(path)) => path,
        _ => "",
      };
      let combined = format!("{path}/{name}");
      combined.replace("/./", "/")
    }
  }
}
