use napi_derive::napi;

/// Dummy loader / resolver placeholder NAPI module for QuickJS module loading customization.
#[napi(js_name = "Loader")]
pub struct JsLoader;

#[napi]
impl JsLoader {
  #[napi(constructor)]
  pub fn new() -> Self {
    JsLoader
  }
}
