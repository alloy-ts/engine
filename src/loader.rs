#[cfg(feature = "quickjs")]
use std::collections::HashMap;
#[cfg(feature = "quickjs")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "quickjs")]
use napi_derive::napi;
#[cfg(feature = "quickjs")]
use rquickjs::{
    loader::{ImportAttributes, Loader, Resolver},
    Ctx, Error, Module, Result as QjsResult,
};

/// The shared module registry: specifier -> module source text.
#[cfg(feature = "quickjs")]
#[derive(Default)]
struct LoaderData {
    modules: HashMap<String, String>,
}

#[cfg(feature = "quickjs")]
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

#[cfg(feature = "quickjs")]
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

#[cfg(feature = "quickjs")]
#[derive(Clone)]
pub(crate) struct SharedModules(Arc<Mutex<LoaderData>>);

#[cfg(feature = "quickjs")]
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

#[cfg(feature = "quickjs")]
impl Loader for SharedModules {
    fn load<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        name: &str,
        _attributes: Option<ImportAttributes<'js>>,
    ) -> QjsResult<Module<'js>> {
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

#[cfg(feature = "quickjs")]
#[napi]
pub struct JsLoader {
    inner: Arc<Mutex<LoaderData>>,
}

#[cfg(feature = "quickjs")]
#[napi]
impl JsLoader {
    #[napi(constructor)]
    pub fn new() -> Self {
        JsLoader {
            inner: Arc::new(Mutex::new(LoaderData::default())),
        }
    }

    #[napi]
    pub fn add(&self, name: String, source: String) {
        self.inner.lock().unwrap().modules.insert(name, source);
    }

    #[napi]
    pub fn remove(&self, name: String) -> bool {
        self.inner.lock().unwrap().modules.remove(&name).is_some()
    }

    #[napi]
    pub fn has(&self, name: String) -> bool {
        self.inner.lock().unwrap().modules.contains_key(&name)
    }

    #[napi]
    pub fn keys(&self) -> Vec<String> {
        self.inner.lock().unwrap().modules.keys().cloned().collect()
    }

    #[napi]
    pub fn clear(&self) {
        self.inner.lock().unwrap().modules.clear();
    }
}

#[cfg(feature = "quickjs")]
impl JsLoader {
    pub(crate) fn resolver(&self) -> SharedModules {
        SharedModules(self.inner.clone())
    }

    pub(crate) fn loader(&self) -> SharedModules {
        SharedModules(self.inner.clone())
    }
}
