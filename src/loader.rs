use napi_derive::napi;
use rquickjs::loader::{BuiltinLoader, BuiltinResolver, FileResolver, ModuleLoader, ScriptLoader};

/// NAPI wrapper for configuring module resolution and loading on a QuickJS Runtime.
#[napi(js_name = "Loader")]
pub struct JsLoader {
    /// Paths to search for module files.
    pub(crate) search_paths: Vec<String>,
    /// Custom string-based built-in modules: module name -> source code.
    pub(crate) builtins: Vec<(String, String)>,
}

#[napi]
impl JsLoader {
    /// Create a new module loader configuration.
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        Ok(Self {
            search_paths: Vec::new(),
            builtins: Vec::new(),
        })
    }

    /// Add a directory path to search when resolving file modules.
    #[napi(js_name = "add_search_path")]
    pub fn add_search_path(&mut self, path: String) {
        self.search_paths.push(path);
    }

    /// Register an in-memory builtin module with a given name and source string.
    #[napi(js_name = "add_builtin")]
    pub fn add_builtin(&mut self, name: String, source: String) {
        self.builtins.push((name, source));
    }
}

impl Default for JsLoader {
    fn default() -> Self {
        Self {
            search_paths: Vec::new(),
            builtins: Vec::new(),
        }
    }
}

/// Attach the configured loader to a QuickJS runtime.
pub fn apply_loader_to_runtime(runtime: &rquickjs::Runtime, loader_cfg: &JsLoader) {
    let mut file_resolver = FileResolver::default();
    for path in &loader_cfg.search_paths {
        file_resolver.add_path(path.as_str());
    }

    let mut builtin_resolver = BuiltinResolver::default();
    let mut builtin_loader = BuiltinLoader::default();

    for (name, source) in &loader_cfg.builtins {
        builtin_resolver.add_module(name.clone());
        builtin_loader.add_module(name.clone(), source.clone());
    }

    let resolver = (file_resolver, builtin_resolver);
    let loader = (builtin_loader, ScriptLoader::default(), ModuleLoader::default());

    runtime.set_loader(resolver, loader);
}
