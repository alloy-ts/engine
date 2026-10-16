//! Context and Ctx types and NAPI bindings.

use napi::bindgen_prelude::{FromNapiValue, JsObjectValue, Object, ToNapiValue, Unknown};
use napi::{Env, Result as NapiResult};
use napi_derive::napi;

use crate::runtime::{JSRuntime, Runtime};

pub use rquickjs::context::intrinsic;
pub use rquickjs::context::ContextBuilder;
pub use rquickjs::context::EvalOptions;
pub use rquickjs::context::Intrinsic;
pub use rquickjs::qjs::JSContext;
pub use rquickjs::Ctx;

/// NAPI object for EvalOptions.
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct JsEvalOptions {
  pub global: Option<bool>,
  pub strict: Option<bool>,
  pub backtrace_barrier: Option<bool>,
  pub promise: Option<bool>,
  pub filename: Option<String>,
}

impl From<JsEvalOptions> for EvalOptions {
  fn from(opts: JsEvalOptions) -> Self {
    let mut eval_opts = EvalOptions::default();
    eval_opts.global = opts.global.unwrap_or(true);
    eval_opts.strict = opts.strict.unwrap_or(true);
    eval_opts.backtrace_barrier = opts.backtrace_barrier.unwrap_or(false);
    eval_opts.promise = opts.promise.unwrap_or(false);
    eval_opts.filename = opts.filename;
    eval_opts
  }
}

fn get_undefined<'env>(env: &'env Env) -> NapiResult<Unknown<'env>> {
  let mut raw = std::ptr::null_mut();
  napi::check_status!(
    unsafe { napi::sys::napi_get_undefined(env.raw(), &mut raw) },
    "Failed to get undefined"
  )?;
  unsafe { Unknown::from_napi_value(env.raw(), raw) }
}

/// A single execution context with its own global variables and stack.
#[napi(js_name = "Context")]
#[derive(Clone)]
pub struct Context {
  pub(crate) inner: rquickjs::Context,
  pub(crate) runtime: Runtime,
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl Context {
  pub unsafe fn from_raw(ctx: std::ptr::NonNull<JSContext>, rt: Runtime) -> Self {
    let inner = unsafe { rquickjs::Context::from_raw(ctx, rt.inner.clone()) };
    Self { inner, runtime: rt }
  }

  pub fn as_raw(&self) -> std::ptr::NonNull<JSContext> {
    self.inner.as_raw()
  }

  pub fn base(runtime: &Runtime) -> NapiResult<Self> {
    let inner = rquickjs::Context::base(&runtime.inner)
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(Self {
      inner,
      runtime: runtime.clone(),
    })
  }

  pub fn custom<I: Intrinsic>(runtime: &Runtime) -> NapiResult<Self> {
    let inner = rquickjs::Context::custom::<I>(&runtime.inner)
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(Self {
      inner,
      runtime: runtime.clone(),
    })
  }

  pub fn full(runtime: &Runtime) -> NapiResult<Self> {
    let inner = rquickjs::Context::full(&runtime.inner)
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(Self {
      inner,
      runtime: runtime.clone(),
    })
  }

  pub fn builder() -> ContextBuilder<()> {
    rquickjs::Context::builder()
  }

  pub fn runtime(&self) -> &Runtime {
    &self.runtime
  }

  pub fn get_runtime_ptr(&self) -> *mut JSRuntime {
    self.inner.get_runtime_ptr()
  }

  pub fn with<F, R>(&self, f: F) -> R
  where
    F: FnOnce(Ctx<'_>) -> R,
  {
    self.inner.with(f)
  }
}

#[napi]
impl Context {
  #[napi(constructor)]
  pub fn new() -> napi::Result<Self> {
    let rt = Runtime::new()?;
    Context::full(&rt)
  }

  #[napi(factory, js_name = "createBase")]
  pub fn create_base() -> napi::Result<Self> {
    let rt = Runtime::new()?;
    Context::base(&rt)
  }

  #[napi(factory, js_name = "createFull")]
  pub fn create_full() -> napi::Result<Self> {
    let rt = Runtime::new()?;
    Context::full(&rt)
  }

  #[napi]
  pub fn eval<'env>(&self, env: &'env Env, source: String) -> NapiResult<Unknown<'env>> {
    self.eval_internal(env, &source, EvalOptions::default())
  }

  #[napi]
  pub fn eval_with_options<'env>(
    &self,
    env: &'env Env,
    source: String,
    options: Option<JsEvalOptions>,
  ) -> NapiResult<Unknown<'env>> {
    let eval_opts: EvalOptions = options.map(Into::into).unwrap_or_default();
    self.eval_internal(env, &source, eval_opts)
  }

  fn eval_internal<'env>(
    &self,
    env: &'env Env,
    source: &str,
    options: EvalOptions,
  ) -> NapiResult<Unknown<'env>> {
    let json_str_opt: Option<String> = self.with(|ctx| -> NapiResult<Option<String>> {
      let val: rquickjs::Value = ctx.eval_with_options(source, options).map_err(|e| {
        if ctx.has_exception() {
          let exc = ctx.catch();
          let msg = ctx
            .json_stringify(&exc)
            .ok()
            .flatten()
            .and_then(|s| s.to_string().ok())
            .unwrap_or_else(|| e.to_string());
          napi::Error::from_reason(msg)
        } else {
          napi::Error::from_reason(e.to_string())
        }
      })?;
      if val.is_undefined() {
        return Ok(None);
      }
      let json = ctx
        .json_stringify(val)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
      Ok(json.and_then(|s| s.to_string().ok()))
    })?;

    if let Some(json_str) = json_str_opt {
      let json_val = env.create_string(&json_str)?;
      let global = env.get_global()?;
      let json_obj: Object = global.get_named_property("JSON")?;
      let parse_fn: napi::bindgen_prelude::Function<Unknown, Unknown> =
        json_obj.get_named_property("parse")?;
      parse_fn.apply(json_obj, json_val.into_unknown(env)?)
    } else {
      get_undefined(env)
    }
  }

  #[napi]
  pub fn json_parse<'env>(&self, env: &'env Env, json: String) -> NapiResult<Unknown<'env>> {
    let json_str_opt: Option<String> = self.with(|ctx| -> NapiResult<Option<String>> {
      let val = ctx.json_parse(json.as_str()).map_err(|e| napi::Error::from_reason(e.to_string()))?;
      let json_res = ctx.json_stringify(val).map_err(|e| napi::Error::from_reason(e.to_string()))?;
      Ok(json_res.and_then(|s| s.to_string().ok()))
    })?;

    if let Some(res) = json_str_opt {
      let json_val = env.create_string(&res)?;
      let global = env.get_global()?;
      let json_obj: Object = global.get_named_property("JSON")?;
      let parse_fn: napi::bindgen_prelude::Function<Unknown, Unknown> =
        json_obj.get_named_property("parse")?;
      parse_fn.apply(json_obj, json_val.into_unknown(env)?)
    } else {
      get_undefined(env)
    }
  }

  #[napi]
  pub fn json_stringify(&self, env: &Env, value: Unknown) -> NapiResult<Option<String>> {
    let global = env.get_global()?;
    let json_obj: Object = global.get_named_property("JSON")?;
    let stringify_fn: napi::bindgen_prelude::Function<Unknown, String> =
      json_obj.get_named_property("stringify")?;
    let node_json = stringify_fn.apply(json_obj, value)?;

    let json_str: Option<String> = self.with(|ctx| -> NapiResult<Option<String>> {
      let val = ctx.json_parse(node_json.as_str()).map_err(|e| napi::Error::from_reason(e.to_string()))?;
      let res = ctx.json_stringify(val).map_err(|e| napi::Error::from_reason(e.to_string()))?;
      Ok(res.and_then(|s| s.to_string().ok()))
    })?;

    Ok(json_str)
  }

  #[napi]
  pub fn run_gc(&self) {
    self.runtime.run_gc();
  }

  #[napi]
  pub fn has_exception(&self) -> bool {
    self.with(|ctx| ctx.has_exception())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn basic() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::full(&rt).unwrap();
    ctx.with(|ctx| {
      let val: i32 = ctx.eval("1+1").unwrap();
      assert_eq!(val, 2);
    });
  }

  #[test]
  fn minimal() {
    let rt = Runtime::new().unwrap();
    rt.set_max_stack_size(0);
    let ctx = Context::builder()
      .with::<intrinsic::Eval>()
      .build(&rt.inner)
      .unwrap();
    let context = Context {
      inner: ctx,
      runtime: rt,
    };
    context.with(|ctx| {
      let val: i32 = ctx.eval("1+1").unwrap();
      assert_eq!(val, 2);
    });
  }

  #[test]
  fn base() {
    let rt = Runtime::new().unwrap();
    let _ = Context::base(&rt).unwrap();
  }

  #[test]
  fn clone_ctx() {
    let rt = Runtime::new().unwrap();
    rt.set_max_stack_size(0);
    let ctx = Context::builder()
      .with::<intrinsic::Eval>()
      .build(&rt.inner)
      .unwrap();

    let context = Context {
      inner: ctx,
      runtime: rt,
    };

    let ctx_clone = context.clone();

    context.with(|ctx| {
      let val: i32 = ctx.eval("1+1").unwrap();
      assert_eq!(val, 2);
    });

    ctx_clone.with(|ctx| {
      let val: i32 = ctx.eval("1+1").unwrap();
      assert_eq!(val, 2);
    });
  }

  #[test]
  fn parallel() {
    use std::thread;

    let rt = Runtime::new().unwrap();
    rt.set_max_stack_size(0);
    let ctx = Context::full(&rt).unwrap();
    thread::spawn(move || {
      ctx.with(|ctx| {
        let val: i32 = ctx.eval("1+1").unwrap();
        assert_eq!(val, 2);
      });
    })
    .join()
    .unwrap();
  }

  #[test]
  fn parallel_drop() {
    use std::sync::{Arc, Barrier};
    use std::thread;

    let wait_for_entry = Arc::new(Barrier::new(2));

    let rt = Runtime::new().unwrap();
    let ctx_1 = Context::full(&rt).unwrap();
    let ctx_2 = Context::full(&rt).unwrap();
    let wait_for_entry_c = wait_for_entry.clone();
    thread::spawn(move || {
      wait_for_entry_c.wait();
      std::mem::drop(ctx_1);
    });

    ctx_2.with(|_ctx| {
      wait_for_entry.wait();
    });
  }
}
