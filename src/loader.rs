//! NAPI bindings for QuickJS Module Loader.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::loader::{BuiltinResolver, ScriptLoader};
use crate::engine::Engine;

/// NAPI wrapper around QuickJS module loader.
#[napi(js_name = "Loader")]
pub struct Loader;

#[napi]
impl Loader {
  /// Set default module loader on `engine`.
  #[napi(js_name = "set_loader")]
  pub fn set_loader(engine: &Engine) -> Result<()> {
    if let Some(rt) = &engine.qjs_runtime {
      let resolver = BuiltinResolver::default();
      let loader = ScriptLoader::default();
      rt.set_loader(resolver, loader);
      Ok(())
    } else {
      Err(Error::from_reason("No QuickJS runtime available"))
    }
  }
}
