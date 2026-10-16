//! Module loader and resolver interface and NAPI bindings.

use napi_derive::napi;
use crate::context::Ctx;

/// Import attributes associated with module imports.
#[derive(Debug, Clone, Default)]
pub struct ImportAttributes<'js> {
  _marker: std::marker::PhantomData<&'js ()>,
}

/// Module loader interface.
pub trait Loader {
  fn load<'js>(
    &mut self,
    ctx: &Ctx<'js>,
    name: &str,
    attributes: Option<ImportAttributes<'js>>,
  ) -> Result<String, napi::Error>;
}

/// Module resolver interface.
pub trait Resolver {
  fn resolve<'js>(
    &mut self,
    ctx: &Ctx<'js>,
    base: &str,
    name: &str,
    attributes: Option<ImportAttributes<'js>>,
  ) -> Result<String, napi::Error>;
}

fn resolve_path(base: &str, name: &str) -> String {
  if !name.starts_with('.') {
    name.to_string()
  } else {
    let mut split = base.rsplitn(2, '/');
    let path = match (split.next(), split.next()) {
      (_, Some(path)) => path,
      _ => "",
    };
    if path.is_empty() {
      name.to_string()
    } else {
      let clean_name = name.strip_prefix("./").unwrap_or(name);
      format!("{path}/{clean_name}")
    }
  }
}

impl Loader for () {
  fn load<'js>(
    &mut self,
    _ctx: &Ctx<'js>,
    name: &str,
    _attributes: Option<ImportAttributes<'js>>,
  ) -> Result<String, napi::Error> {
    Err(napi::Error::from_reason(format!("Module not found: {name}")))
  }
}

impl Resolver for () {
  fn resolve<'js>(
    &mut self,
    _ctx: &Ctx<'js>,
    base: &str,
    name: &str,
    _attributes: Option<ImportAttributes<'js>>,
  ) -> Result<String, napi::Error> {
    Ok(resolve_path(base, name))
  }
}

/// Resolver for built-in modules.
#[napi(js_name = "BuiltinResolver")]
#[derive(Debug, Clone, Default)]
pub struct BuiltinResolver;

#[napi]
impl BuiltinResolver {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self
  }
}

impl Resolver for BuiltinResolver {
  fn resolve<'js>(
    &mut self,
    _ctx: &Ctx<'js>,
    _base: &str,
    name: &str,
    _attributes: Option<ImportAttributes<'js>>,
  ) -> Result<String, napi::Error> {
    Ok(name.to_string())
  }
}

/// Loader for built-in modules.
#[napi(js_name = "BuiltinLoader")]
#[derive(Debug, Clone, Default)]
pub struct BuiltinLoader;

#[napi]
impl BuiltinLoader {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self
  }
}

impl Loader for BuiltinLoader {
  fn load<'js>(
    &mut self,
    _ctx: &Ctx<'js>,
    name: &str,
    _attributes: Option<ImportAttributes<'js>>,
  ) -> Result<String, napi::Error> {
    Ok(format!("export default {};", name))
  }
}

/// Resolver for file system paths.
#[napi(js_name = "FileResolver")]
#[derive(Debug, Clone, Default)]
pub struct FileResolver;

#[napi]
impl FileResolver {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self
  }
}

impl Resolver for FileResolver {
  fn resolve<'js>(
    &mut self,
    _ctx: &Ctx<'js>,
    base: &str,
    name: &str,
    _attributes: Option<ImportAttributes<'js>>,
  ) -> Result<String, napi::Error> {
    Ok(resolve_path(base, name))
  }
}

/// Loader for scripts.
#[napi(js_name = "ScriptLoader")]
#[derive(Debug, Clone, Default)]
pub struct ScriptLoader;

#[napi]
impl ScriptLoader {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self
  }
}

impl Loader for ScriptLoader {
  fn load<'js>(
    &mut self,
    _ctx: &Ctx<'js>,
    name: &str,
    _attributes: Option<ImportAttributes<'js>>,
  ) -> Result<String, napi::Error> {
    Ok(format!("// script loader for {name}"))
  }
}

/// Native dynamic library loader.
#[napi(js_name = "NativeLoader")]
#[derive(Debug, Clone, Default)]
pub struct NativeLoader;

#[napi]
impl NativeLoader {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self
  }
}

impl Loader for NativeLoader {
  fn load<'js>(
    &mut self,
    _ctx: &Ctx<'js>,
    name: &str,
    _attributes: Option<ImportAttributes<'js>>,
  ) -> Result<String, napi::Error> {
    Ok(format!("// native loader for {name}"))
  }
}

/// General ES module loader.
#[napi(js_name = "ModuleLoader")]
#[derive(Debug, Clone, Default)]
pub struct ModuleLoader;

#[napi]
impl ModuleLoader {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self
  }
}

impl Loader for ModuleLoader {
  fn load<'js>(
    &mut self,
    _ctx: &Ctx<'js>,
    name: &str,
    _attributes: Option<ImportAttributes<'js>>,
  ) -> Result<String, napi::Error> {
    Ok(format!("// module loader for {name}"))
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_default_resolver() {
    let mut resolver = FileResolver::new();
    let ctx = crate::context::Ctx::dangling();
    let res = resolver.resolve(&ctx, "src/main.js", "./util.js", None).unwrap();
    assert_eq!(res, "src/util.js");
  }
}
