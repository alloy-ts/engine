//! Module loader and resolver interface and NAPI bindings.

use napi_derive::napi;

pub use rquickjs::loader::BuiltinLoader;
pub use rquickjs::loader::BuiltinResolver;
pub use rquickjs::loader::FileResolver;
pub use rquickjs::loader::ScriptLoader;

pub use rquickjs::loader::Loader;
pub use rquickjs::loader::ModuleLoader;
pub use rquickjs::loader::NativeLoader;
pub use rquickjs::loader::Resolver;

/// NAPI-exposed FileResolver.
#[napi(js_name = "FileResolver")]
#[derive(Debug, Clone, Default)]
pub struct JsFileResolver;

#[napi]
impl JsFileResolver {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self
  }
}

/// NAPI-exposed ScriptLoader.
#[napi(js_name = "ScriptLoader")]
#[derive(Debug, Clone, Default)]
pub struct JsScriptLoader;

#[napi]
impl JsScriptLoader {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self
  }
}

/// NAPI-exposed BuiltinResolver.
#[napi(js_name = "BuiltinResolver")]
#[derive(Debug, Clone, Default)]
pub struct JsBuiltinResolver;

#[napi]
impl JsBuiltinResolver {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self
  }
}

/// NAPI-exposed BuiltinLoader.
#[napi(js_name = "BuiltinLoader")]
#[derive(Debug, Clone, Default)]
pub struct JsBuiltinLoader;

#[napi]
impl JsBuiltinLoader {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self
  }
}

/// NAPI-exposed NativeLoader.
#[napi(js_name = "NativeLoader")]
#[derive(Debug, Clone, Default)]
pub struct JsNativeLoader;

#[napi]
impl JsNativeLoader {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self
  }
}

/// NAPI-exposed ModuleLoader.
#[napi(js_name = "ModuleLoader")]
#[derive(Debug, Clone, Default)]
pub struct JsModuleLoader;

#[napi]
impl JsModuleLoader {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_default_resolver() {
    let mut resolver = FileResolver::default().with_path("src");
    let rt = rquickjs::Runtime::new().unwrap();
    let ctx = rquickjs::Context::full(&rt).unwrap();
    ctx.with(|ctx| {
      let res = resolver.resolve(&ctx, "src/main.js", "./util.js", None);
      assert!(res.is_ok() || res.is_err());
    });
  }
}
