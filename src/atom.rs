use napi_derive::napi;

/// A QuickJS Atom wrapper.
#[napi(js_name = "Atom")]
pub struct JsAtom {
  pub(crate) name: String,
}

#[napi]
impl JsAtom {
  #[napi(constructor)]
  pub fn new(name: String) -> Self {
    Self { name }
  }

  #[napi]
  pub fn to_string(&self) -> String {
    self.name.clone()
  }
}
