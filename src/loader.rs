use napi_derive::napi;

/// Module Loader configuration for QuickJS runtime.
#[napi]
pub struct Loader {
  pub(crate) name: String,
}

#[napi]
impl Loader {
  #[napi(constructor)]
  pub fn new(name: Option<String>) -> Self {
    Self {
      name: name.unwrap_or_else(|| "default-loader".to_string()),
    }
  }

  #[napi]
  pub fn get_name(&self) -> String {
    self.name.clone()
  }
}
