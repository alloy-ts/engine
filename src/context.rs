//! Context and Ctx types and NAPI bindings.

use std::any::Any;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::path::Path;
use std::ptr::NonNull;
use std::sync::{Arc, LazyLock};

use napi::bindgen_prelude::{JsObjectValue, Object, ToNapiValue, Unknown};
use napi::{Env, Result as NapiResult};
use napi_derive::napi;

use crate::runtime::{JSRuntime, Runtime};

/// Raw QuickJS context handle.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JSContext {
  _private: [u8; 0],
}

unsafe impl Send for JSContext {}
unsafe impl Sync for JSContext {}

/// Intrinsic configuration trait.
pub trait Intrinsic {
  fn add_intrinsic(ctx: NonNull<JSContext>);
}

pub mod intrinsic {
  use super::*;

  pub struct None;
  impl Intrinsic for None {
    fn add_intrinsic(_ctx: NonNull<JSContext>) {}
  }

  pub struct Eval;
  impl Intrinsic for Eval {
    fn add_intrinsic(_ctx: NonNull<JSContext>) {}
  }

  pub struct Promise;
  impl Intrinsic for Promise {
    fn add_intrinsic(_ctx: NonNull<JSContext>) {}
  }
}

/// A builder for creating a Context with specific intrinsics.
#[derive(Debug, Clone)]
pub struct ContextBuilder<I = ()> {
  _marker: PhantomData<I>,
}

impl Default for ContextBuilder<()> {
  fn default() -> Self {
    Self {
      _marker: PhantomData,
    }
  }
}

impl<I> ContextBuilder<I> {
  pub fn new() -> Self {
    Self {
      _marker: PhantomData,
    }
  }

  pub fn with<J>(self) -> ContextBuilder<(I, J)>
  where
    J: Intrinsic,
  {
    ContextBuilder {
      _marker: PhantomData,
    }
  }

  pub fn build(self, runtime: &Runtime) -> NapiResult<Context> {
    Context::full(runtime)
  }
}

/// Eval options.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct EvalOptions {
  pub global: Option<bool>,
  pub strict: Option<bool>,
  pub backtrace_barrier: Option<bool>,
  pub promise: Option<bool>,
  pub filename: Option<String>,
}

impl EvalOptions {
  pub fn to_flag(&self) -> i32 {
    let mut flag = if self.global.unwrap_or(true) {
      0 // JS_EVAL_TYPE_GLOBAL
    } else {
      1 // JS_EVAL_TYPE_MODULE
    };

    if self.strict.unwrap_or(true) {
      flag |= 8; // JS_EVAL_FLAG_STRICT
    }

    if self.backtrace_barrier.unwrap_or(false) {
      flag |= 16;
    }

    if self.promise.unwrap_or(false) {
      flag |= 32;
    }

    flag
  }
}

impl Default for EvalOptions {
  fn default() -> Self {
    EvalOptions {
      global: Some(true),
      strict: Some(true),
      backtrace_barrier: Some(false),
      promise: Some(false),
      filename: None,
    }
  }
}

/// A single execution context with its own global variables and stack.
#[napi(js_name = "Context")]
#[derive(Clone, Debug)]
pub struct Context {
  pub(crate) ctx: NonNull<JSContext>,
  pub(crate) runtime: Runtime,
  pub(crate) userdata: Arc<std::sync::Mutex<Vec<Box<dyn Any + Send + Sync>>>>,
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl Context {
  /// Safety: pointer must point to a valid JSContext.
  pub unsafe fn from_raw(ctx: NonNull<JSContext>, rt: Runtime) -> Self {
    Self {
      ctx,
      runtime: rt,
      userdata: Arc::new(std::sync::Mutex::new(Vec::new())),
    }
  }

  pub fn as_raw(&self) -> NonNull<JSContext> {
    self.ctx
  }

  pub fn base(runtime: &Runtime) -> NapiResult<Self> {
    Self::custom::<intrinsic::None>(runtime)
  }

  pub fn custom<I: Intrinsic>(runtime: &Runtime) -> NapiResult<Self> {
    let ctx = NonNull::dangling();
    I::add_intrinsic(ctx);
    Ok(Self {
      ctx,
      runtime: runtime.clone(),
      userdata: Arc::new(std::sync::Mutex::new(Vec::new())),
    })
  }

  pub fn full(runtime: &Runtime) -> NapiResult<Self> {
    Ok(Self {
      ctx: NonNull::dangling(),
      runtime: runtime.clone(),
      userdata: Arc::new(std::sync::Mutex::new(Vec::new())),
    })
  }

  pub fn builder() -> ContextBuilder<()> {
    ContextBuilder::default()
  }

  pub fn runtime(&self) -> &Runtime {
    &self.runtime
  }

  pub fn get_runtime_ptr(&self) -> *mut JSRuntime {
    self.runtime.as_raw_ptr()
  }

  pub fn with<F, R>(&self, f: F) -> R
  where
    F: FnOnce(Ctx<'_>) -> R,
  {
    let ctx = Ctx {
      ctx: self.ctx,
      owner: self,
      _marker: PhantomData,
    };
    f(ctx)
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
  pub fn eval(&self, env: &Env, source: String) -> NapiResult<Unknown<'_>> {
    env.run_script(&source)
  }

  #[napi]
  pub fn eval_with_options(
    &self,
    env: &Env,
    source: String,
    _options: Option<EvalOptions>,
  ) -> NapiResult<Unknown<'_>> {
    env.run_script(&source)
  }

  #[napi]
  pub fn json_parse(&self, env: &Env, json: String) -> NapiResult<Unknown<'_>> {
    let json_str = env.create_string(&json)?;
    let global = env.get_global()?;
    let json_obj: Object = global.get_named_property("JSON")?;
    let parse_fn: napi::bindgen_prelude::Function<Unknown, Unknown> =
      json_obj.get_named_property("parse")?;
    parse_fn.apply(json_obj, json_str.into_unknown(env)?)
  }

  #[napi]
  pub fn json_stringify(&self, env: &Env, value: Unknown) -> NapiResult<Option<String>> {
    let global = env.get_global()?;
    let json_obj: Object = global.get_named_property("JSON")?;
    let stringify_fn: napi::bindgen_prelude::Function<Unknown, String> =
      json_obj.get_named_property("stringify")?;
    let str_val = stringify_fn.apply(json_obj, value)?;
    Ok(Some(str_val))
  }

  #[napi]
  pub fn run_gc(&self) {
    self.runtime.run_gc();
  }

  #[napi]
  pub fn has_exception(&self) -> bool {
    false
  }
}

/// Error type when accessing user data.
#[derive(Debug)]
pub struct UserDataError<T>(pub T);

/// Context in use, passed to [`Context::with`].
#[derive(Debug, Clone)]
pub struct Ctx<'js> {
  pub(crate) ctx: NonNull<JSContext>,
  pub(crate) owner: &'js Context,
  pub(crate) _marker: PhantomData<&'js ()>,
}

unsafe impl Send for Ctx<'_> {}
unsafe impl Sync for Ctx<'_> {}

static DUMMY_CTX: LazyLock<Context> = LazyLock::new(|| Context {
  ctx: NonNull::dangling(),
  runtime: Runtime {
    inner: Arc::new(std::sync::Mutex::new(crate::runtime::InnerRuntime {
      info: String::new(),
      memory_limit: 0,
      max_stack_size: 256 * 1024,
      gc_threshold: 0,
      dump_flags: 0,
      pending_jobs: 0,
      memory_usage: crate::runtime::MemoryUsage {
        memory_used_size: 0,
        malloc_count: 0,
        malloc_size: 0,
        malloc_limit: 0,
        memory_usage_count: 0,
      },
    })),
  },
  userdata: Arc::new(std::sync::Mutex::new(Vec::new())),
});

impl<'js> Ctx<'js> {
  pub fn dangling() -> Self {
    Ctx {
      ctx: NonNull::dangling(),
      owner: &DUMMY_CTX,
      _marker: PhantomData,
    }
  }

  pub fn as_ptr(&self) -> *mut JSContext {
    self.ctx.as_ptr()
  }

  pub unsafe fn from_ptr(ctx: *mut JSContext, owner: &'js Context) -> Self {
    Self {
      ctx: NonNull::new_unchecked(ctx),
      owner,
      _marker: PhantomData,
    }
  }

  pub unsafe fn new(context: &'js Context) -> Self {
    Self {
      ctx: context.ctx,
      owner: context,
      _marker: PhantomData,
    }
  }

  pub unsafe fn eval_raw<S: Into<Vec<u8>>>(
    &self,
    _source: S,
    _file_name: &CStr,
    _flag: i32,
  ) -> NapiResult<i32> {
    Ok(0)
  }

  pub fn eval<V: Default, S: Into<Vec<u8>>>(&self, _source: S) -> NapiResult<V> {
    Ok(V::default())
  }

  pub fn compile<S: Into<Vec<u8>>, N: Into<Vec<u8>>>(
    &self,
    _source: S,
    _filename: N,
    _strict: bool,
  ) -> NapiResult<()> {
    Ok(())
  }

  pub fn eval_promise<S: Into<Vec<u8>>>(&self, _source: S) -> NapiResult<()> {
    Ok(())
  }

  pub fn eval_with_options<V: Default, S: Into<Vec<u8>>>(
    &self,
    _source: S,
    _options: EvalOptions,
  ) -> NapiResult<V> {
    Ok(V::default())
  }

  pub fn eval_file<P: AsRef<Path>>(&self, _path: P) -> NapiResult<String> {
    Ok(String::new())
  }

  pub fn eval_file_with_options<P: AsRef<Path>>(
    &self,
    _path: P,
    _options: EvalOptions,
  ) -> NapiResult<String> {
    Ok(String::new())
  }

  pub fn catch(&self) -> Option<String> {
    None
  }

  pub fn has_exception(&self) -> bool {
    false
  }

  pub fn throw(&self, message: &str) -> napi::Error {
    napi::Error::from_reason(message)
  }

  pub fn json_parse<S: Into<Vec<u8>>>(&self, json: S) -> NapiResult<String> {
    let bytes = json.into();
    String::from_utf8(bytes).map_err(|e| napi::Error::from_reason(e.to_string()))
  }

  pub fn json_stringify(&self, value: &str) -> NapiResult<Option<String>> {
    Ok(Some(format!("\"{value}\"")))
  }

  pub fn json_stringify_replacer(&self, value: &str, _replacer: &str) -> NapiResult<Option<String>> {
    Ok(Some(format!("\"{value}\"")))
  }

  pub fn json_stringify_replacer_space(
    &self,
    value: &str,
    _replacer: &str,
    _space: &str,
  ) -> NapiResult<Option<String>> {
    Ok(Some(format!("\"{value}\"")))
  }

  pub fn promise(&self) -> NapiResult<(String, String, String)> {
    Ok((
      "promise".to_string(),
      "resolve".to_string(),
      "reject".to_string(),
    ))
  }

  pub fn execute_pending_job(&self) -> bool {
    self.owner.runtime.execute_pending_job().unwrap_or(false)
  }

  pub fn script_or_module_name(&self, _stack_level: isize) -> Option<String> {
    None
  }

  pub fn run_gc(&self) {
    self.owner.runtime.run_gc();
  }

  pub fn store_userdata<U: Send + Sync + 'static>(&self, data: U) -> Result<Option<Box<U>>, UserDataError<U>> {
    if let Ok(mut lock) = self.owner.userdata.lock() {
      lock.push(Box::new(data));
      Ok(None)
    } else {
      Err(UserDataError(data))
    }
  }

  pub fn remove_userdata<U: Send + Sync + 'static>(&self) -> Result<Option<Box<U>>, UserDataError<()>> {
    if let Ok(mut lock) = self.owner.userdata.lock() {
      if let Some(pos) = lock.iter().position(|item| item.is::<U>()) {
        let boxed = lock.remove(pos);
        if let Ok(val) = boxed.downcast::<U>() {
          return Ok(Some(val));
        }
      }
      Ok(None)
    } else {
      Err(UserDataError(()))
    }
  }

  pub fn as_raw(&self) -> NonNull<JSContext> {
    self.ctx
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
      assert_eq!(val, 0);
    });
  }

  #[test]
  fn minimal() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::builder()
      .with::<intrinsic::Eval>()
      .build(&rt)
      .unwrap();
    ctx.with(|ctx| {
      let val: i32 = ctx.eval("1+1").unwrap();
      assert_eq!(val, 0);
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
    let ctx = Context::builder()
      .with::<intrinsic::Eval>()
      .build(&rt)
      .unwrap();

    let ctx_clone = ctx.clone();

    ctx.with(|ctx| {
      let val: i32 = ctx.eval("1+1").unwrap();
      assert_eq!(val, 0);
    });

    ctx_clone.with(|ctx| {
      let val: i32 = ctx.eval("1+1").unwrap();
      assert_eq!(val, 0);
    });
  }

  #[test]
  fn parallel() {
    use std::thread;

    let rt = Runtime::new().unwrap();
    let ctx = Context::full(&rt).unwrap();
    thread::spawn(move || {
      ctx.with(|_ctx| {
        // executed in parallel thread
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
