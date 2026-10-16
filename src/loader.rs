use napi_derive::napi;

/// Options and configuration for module loading and resolution.
#[napi(object)]
pub struct LoaderOptions {
  pub base_dir: Option<String>,
}

/// NAPI wrapper around module resolution and loading helpers for JS engine.
#[napi]
pub struct Loader {
  base_dir: Option<String>,
}

#[napi]
impl Loader {
  #[napi(constructor)]
  pub fn new(options: Option<LoaderOptions>) -> Self {
    Self {
      base_dir: options.and_then(|o| o.base_dir),
    }
  }

  /// Resolve a module specifier relative to a base path.
  #[napi]
  pub fn resolve(&self, base: String, name: String) -> String {
    if name.starts_with('.') || name.starts_with('/') {
      if let Some(parent) = std::path::Path::new(&base).parent() {
        return parent.join(&name).to_string_lossy().to_string();
      }
    }
    if let Some(ref base_dir) = self.base_dir {
      return std::path::Path::new(base_dir).join(&name).to_string_lossy().to_string();
    }
    name
  }
}
