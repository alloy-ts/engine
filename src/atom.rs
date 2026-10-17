//! NAPI bindings for QuickJS Atom (`rquickjs::Atom`).

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::Atom as QjsAtom;

use crate::context::Context;

/// Primitive types exposed for NAPI bindings
#[napi]
pub enum PrimitiveType {
  U32,
  I32,
  U64,
  I64,
  F64,
  F32,
  BOOL,
  STRING,
}

/// JS class wrapping QuickJS `Atom`.
#[napi]
pub struct Atom {
  pub(crate) val_str: String,
}

#[napi]
impl Atom {
  #[napi(constructor)]
  pub fn new(val: String) -> Self {
    Self { val_str: val }
  }

  #[napi(factory, js_name = "from_value")]
  pub fn from_value(ctx: &Context, val: String) -> Result<Self> {
    ctx.inner.with(|qjs_ctx| {
      let atom = QjsAtom::from_str(qjs_ctx, &val).map_err(|e| Error::from_reason(e.to_string()))?;
      let s = atom.to_string().map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { val_str: s })
    })
  }

  #[napi(factory, js_name = "from_u32")]
  pub fn from_u32(ctx: &Context, val: u32) -> Result<Self> {
    ctx.inner.with(|qjs_ctx| {
      let atom = QjsAtom::from_u32(qjs_ctx, val).map_err(|e| Error::from_reason(e.to_string()))?;
      let s = atom.to_string().map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { val_str: s })
    })
  }

  #[napi(factory, js_name = "fromU32")]
  pub fn from_u32_camel(ctx: &Context, val: u32) -> Result<Self> {
    Self::from_u32(ctx, val)
  }

  #[napi(factory, js_name = "from_i32")]
  pub fn from_i32(ctx: &Context, val: i32) -> Result<Self> {
    ctx.inner.with(|qjs_ctx| {
      let atom = QjsAtom::from_i32(qjs_ctx, val).map_err(|e| Error::from_reason(e.to_string()))?;
      let s = atom.to_string().map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { val_str: s })
    })
  }

  #[napi(factory, js_name = "fromI32")]
  pub fn from_i32_camel(ctx: &Context, val: i32) -> Result<Self> {
    Self::from_i32(ctx, val)
  }

  #[napi(factory, js_name = "from_bool")]
  pub fn from_bool(ctx: &Context, val: bool) -> Result<Self> {
    ctx.inner.with(|qjs_ctx| {
      let atom = QjsAtom::from_bool(qjs_ctx, val).map_err(|e| Error::from_reason(e.to_string()))?;
      let s = atom.to_string().map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { val_str: s })
    })
  }

  #[napi(factory, js_name = "fromBool")]
  pub fn from_bool_camel(ctx: &Context, val: bool) -> Result<Self> {
    Self::from_bool(ctx, val)
  }

  #[napi(factory, js_name = "from_f64")]
  pub fn from_f64(ctx: &Context, val: f64) -> Result<Self> {
    ctx.inner.with(|qjs_ctx| {
      let atom = QjsAtom::from_f64(qjs_ctx, val).map_err(|e| Error::from_reason(e.to_string()))?;
      let s = atom.to_string().map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { val_str: s })
    })
  }

  #[napi(factory, js_name = "fromF64")]
  pub fn from_f64_camel(ctx: &Context, val: f64) -> Result<Self> {
    Self::from_f64(ctx, val)
  }

  #[napi(factory, js_name = "from_str")]
  pub fn from_str(ctx: &Context, name: String) -> Result<Self> {
    ctx.inner.with(|qjs_ctx| {
      let atom = QjsAtom::from_str(qjs_ctx, &name).map_err(|e| Error::from_reason(e.to_string()))?;
      let s = atom.to_string().map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { val_str: s })
    })
  }

  #[napi(factory, js_name = "fromStr")]
  pub fn from_str_camel(ctx: &Context, name: String) -> Result<Self> {
    Self::from_str(ctx, name)
  }

  #[napi(factory, js_name = "from_predefined")]
  pub fn from_predefined(_ctx: &Context, name: String) -> Result<Self> {
    Ok(Self { val_str: name })
  }

  #[napi(factory, js_name = "fromPredefined")]
  pub fn from_predefined_camel(ctx: &Context, name: String) -> Result<Self> {
    Self::from_predefined(ctx, name)
  }

  #[napi(js_name = "to_string")]
  pub fn to_string(&self) -> String {
    self.val_str.clone()
  }

  #[napi(js_name = "toString")]
  pub fn to_string_js(&self) -> String {
    self.val_str.clone()
  }
}
