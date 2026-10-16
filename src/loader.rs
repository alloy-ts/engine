//! NAPI bindings for QuickJS module loader and resolver.

use napi_derive::napi;

/// NAPI wrapper for QuickJS module loader and resolver configuration.
#[napi]
#[derive(Default)]
pub struct Loader {
    pub(crate) paths: Vec<String>,
    pub(crate) modules: Vec<(String, String)>,
}

#[napi]
impl Loader {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            modules: Vec::new(),
        }
    }

    /// Add a module search directory path for file resolution.
    #[napi]
    pub fn add_path(&mut self, path: String) {
        self.paths.push(path);
    }

    /// Register a builtin module by name with source text.
    #[napi]
    pub fn add_module(&mut self, name: String, source: String) {
        self.modules.push((name, source));
    }
}
