//! NAPI bindings for module loaders and resolvers in `rquickjs::loader`.

use napi_derive::napi;
use rquickjs::loader::{BuiltinLoader, BuiltinResolver, FileResolver, ScriptLoader};

#[napi(js_name = "BuiltinResolver")]
pub struct JsBuiltinResolver {
    #[allow(dead_code)]
    pub(crate) inner: BuiltinResolver,
}

#[napi]
impl JsBuiltinResolver {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: BuiltinResolver::default(),
        }
    }
}

#[napi(js_name = "BuiltinLoader")]
pub struct JsBuiltinLoader {
    #[allow(dead_code)]
    pub(crate) inner: BuiltinLoader,
}

#[napi]
impl JsBuiltinLoader {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: BuiltinLoader::default(),
        }
    }
}

#[napi(js_name = "FileResolver")]
pub struct JsFileResolver {
    #[allow(dead_code)]
    pub(crate) inner: FileResolver,
}

#[napi]
impl JsFileResolver {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: FileResolver::default(),
        }
    }
}

#[napi(js_name = "ScriptLoader")]
pub struct JsScriptLoader {
    #[allow(dead_code)]
    pub(crate) inner: ScriptLoader,
}

#[napi]
impl JsScriptLoader {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: ScriptLoader::default(),
        }
    }
}
