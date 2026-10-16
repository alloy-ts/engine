//! NAPI bindings for module `Loader` and `Resolver`.

use napi_derive::napi;

/// Module resolver interface.
#[napi]
pub struct Resolver;

#[napi]
impl Resolver {
  #[napi(constructor)]
  pub fn new() -> Self {
    Resolver
  }

  /// Normalize / resolve module name.
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
      let rel_name = name.strip_prefix("./").unwrap_or(&name);
      if path.is_empty() {
        rel_name.to_string()
      } else {
        format!("{path}/{rel_name}")
      }
    }
  }
}

/// Module loader interface.
#[napi]
pub struct Loader;

#[napi]
impl Loader {
  #[napi(constructor)]
  pub fn new() -> Self {
    Loader
  }

  /// Load module by name.
  #[napi]
  pub fn load(&self, name: String) -> String {
    format!("// Module: {name}\nexport default {{}};")
  }
}
