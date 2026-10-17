use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use napi_derive::napi;
use rquickjs::{
    loader::{ImportAttributes, Loader as QjsLoaderTrait, Resolver},
    Ctx, Error, Module, Result as QjsResult,
};

/// The shared module registry: specifier -> module source text.
///
/// Wrapped in `Arc<Mutex<..>>` so the `Resolver`/`Loader` trait objects handed to
/// the runtime share the *same* state as the JS `Loader` object — modules added
/// after `set_loader` are still visible, and the registry outlives the JS handle.
#[derive(Default)]
struct LoaderData {
    modules: HashMap<String, String>,
}

/// Resolve a module specifier against a base module name.
///
/// Bare specifiers (`"math"`) are used verbatim; relative specifiers
/// (`"./x"`, `"../y"`) are resolved against the base module's directory and
/// path-normalized.
fn resolve_path(base: &str, name: &str) -> String {
    if !name.starts_with('.') {
        return name.to_string();
    }
    let dir = match base.rsplit_once('/') {
        Some((d, _)) => d,
        None => "",
    };
    let joined = if dir.is_empty() {
        name.to_string()
    } else {
        format!("{dir}/{name}")
    };
    normalize(&joined)
}

/// Normalize a `/`-separated path, collapsing `.` and `..` segments.
fn normalize(path: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." => match out.last() {
                Some(last) if *last != ".." => {
                    out.pop();
                }
                _ => out.push(".."),
            },
            other => out.push(other),
        }
    }
    let joined = out.join("/");
    if path.starts_with('/') {
        format!("/{joined}")
    } else {
        joined
    }
}

/// The Rust-side `Resolver` + `Loader`, sharing state with a JS `Loader`.
#[derive(Clone)]
pub(crate) struct SharedModules(Arc<Mutex<LoaderData>>);

impl Resolver for SharedModules {
    fn resolve<'js>(
        &mut self,
        _ctx: &Ctx<'js>,
        base: &str,
        name: &str,
        _attributes: Option<ImportAttributes<'js>>,
    ) -> QjsResult<String> {
        let path = resolve_path(base, name);
        let guard = self.0.lock().unwrap();
        if guard.modules.contains_key(&path) {
            Ok(path)
        } else {
            Err(Error::new_resolving(base, name))
        }
    }
}

impl QjsLoaderTrait for SharedModules {
    fn load<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        name: &str,
        _attributes: Option<ImportAttributes<'js>>,
    ) -> QjsResult<Module<'js>> {
        // Clone the source out so the mutex guard is dropped before compiling.
        let source = {
            let guard = self.0.lock().unwrap();
            guard.modules.get(name).cloned()
        };
        match source {
            Some(src) => Module::declare(ctx.clone(), name, src),
            None => Err(Error::new_loading(name)),
        }
    }
}

/// NAPI class wrapping a host-provided QuickJS module loader/resolver.
///
/// JS:
/// ```ts
/// const loader = new Loader();
/// loader.add("math", "export const add = (a, b) => a + b;");
///
/// const engine = new Engine(loader);     // initiate the engine with it, or
/// // engine.set_loader(loader);          // attach it afterwards
/// const ctx = engine.create_context();
/// ctx.eval_module('import { add } from "math"; globalThis.sum = add(2, 3);');
/// ```
#[napi]
pub struct Loader {
    inner: Arc<Mutex<LoaderData>>,
}

#[napi]
impl Loader {
    /// Create an empty loader.
    #[napi(constructor)]
    pub fn new() -> Self {
        Loader {
            inner: Arc::new(Mutex::new(LoaderData::default())),
        }
    }

    /// Register (or replace) a module's source text under `name`.
    #[napi]
    pub fn add(&self, name: String, source: String) {
        self.inner.lock().unwrap().modules.insert(name, source);
    }

    /// Remove a module. Returns `true` if it existed.
    #[napi]
    pub fn remove(&self, name: String) -> bool {
        self.inner.lock().unwrap().modules.remove(&name).is_some()
    }

    /// Whether a module with this name is registered.
    #[napi]
    pub fn has(&self, name: String) -> bool {
        self.inner.lock().unwrap().modules.contains_key(&name)
    }

    /// The registered module names.
    #[napi]
    pub fn keys(&self) -> Vec<String> {
        self.inner.lock().unwrap().modules.keys().cloned().collect()
    }

    /// Remove all registered modules.
    #[napi]
    pub fn clear(&self) {
        self.inner.lock().unwrap().modules.clear();
    }
}

impl Loader {
    /// A [`Resolver`] sharing this loader's registry. `pub(crate)` — only the
    /// runtime needs it, to hand to `set_loader`.
    pub(crate) fn resolver(&self) -> SharedModules {
        SharedModules(self.inner.clone())
    }

    /// A [`QjsLoaderTrait`] sharing this loader's registry. `pub(crate)` — only the
    /// runtime needs it, to hand to `set_loader`.
    pub(crate) fn loader(&self) -> SharedModules {
        SharedModules(self.inner.clone())
    }
}
