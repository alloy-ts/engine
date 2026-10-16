use napi_derive::napi;

/// A QuickJS Atom representation for Node-API.
#[napi]
pub struct Atom {
  pub(crate) atom: u32,
}

#[napi]
impl Atom {
  #[napi(constructor)]
  pub fn new(atom: u32) -> Self {
    Self { atom }
  }

  #[napi(getter)]
  pub fn value(&self) -> u32 {
    self.atom
  }

  #[napi]
  pub fn to_string(&self) -> String {
    format!("Atom({})", self.atom)
  }
}
