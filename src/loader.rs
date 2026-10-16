//! NAPI bindings for QuickJS [`Loader`] and [`Resolver`].

use napi_derive::napi;
use rquickjs::loader::{BuiltinLoader, BuiltinResolver, FileResolver, ScriptLoader};

/// Built-in module resolver for QuickJS.
#[napi(js_name = "BuiltinResolver")]
pub struct BuiltinResolverWrapper {
    pub(crate) inner: Option<BuiltinResolver>,
}

#[napi]
impl BuiltinResolverWrapper {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Some(BuiltinResolver::default()),
        }
    }
}

/// Built-in module loader for QuickJS.
#[napi(js_name = "BuiltinLoader")]
pub struct BuiltinLoaderWrapper {
    pub(crate) inner: Option<BuiltinLoader>,
}

#[napi]
impl BuiltinLoaderWrapper {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Some(BuiltinLoader::default()),
        }
    }
}

/// Script loader for QuickJS.
#[napi(js_name = "ScriptLoader")]
pub struct ScriptLoaderWrapper {
    #[allow(dead_code)]
    pub(crate) inner: Option<ScriptLoader>,
}

#[napi]
impl ScriptLoaderWrapper {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Some(ScriptLoader::default()),
        }
    }
}

/// File resolver for QuickJS.
#[napi(js_name = "FileResolver")]
pub struct FileResolverWrapper {
    #[allow(dead_code)]
    pub(crate) inner: Option<FileResolver>,
}

#[napi]
impl FileResolverWrapper {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Some(FileResolver::default()),
        }
    }
}
