//! `JsEngine`: NAPI class merging N-API context (`napi::Env`) and QuickJS runtime engine.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::Env;
use napi_derive::napi;

pub use rquickjs::qjs::JSRuntime;

/// Memory usage statistics for the QuickJS engine.
#[napi(object)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryUsage {
  pub memory_used_size: i64,
  pub malloc_count: i64,
  pub malloc_size: i64,
  pub malloc_limit: i64,
  pub memory_usage_count: i64,
}

/// JS class wrapping N-API context (`napi::Env`) and QuickJS runtime (`rquickjs::Runtime`).
#[napi(js_name = "Engine")]
#[derive(Clone)]
pub struct JsEngine {
  pub(crate) env: Env,
  pub(crate) disposed: Arc<AtomicBool>,
  pub(crate) qjs_runtime: rquickjs::Runtime,
}

/// A weak handle to the Engine.
#[napi(js_name = "WeakEngine")]
#[derive(Clone)]
pub struct WeakEngine {
  pub(crate) inner: rquickjs::runtime::WeakRuntime,
}

#[napi]
impl WeakEngine {
  /// Upgrade the weak reference to an Engine reference if still alive.
  #[napi(js_name = "try_ref")]
  pub fn try_ref(&self, env: Env) -> Option<JsEngine> {
    self.inner.try_ref().map(|inner| JsEngine {
      env,
      disposed: Arc::new(AtomicBool::new(false)),
      qjs_runtime: inner,
    })
  }

  /// CamelCase alias for try_ref.
  #[napi(js_name = "tryRef")]
  pub fn try_ref_camel(&self, env: Env) -> Option<JsEngine> {
    self.try_ref(env)
  }
}

#[napi]
impl JsEngine {
  /// Capture the `napi::Env` and create a QuickJS `Runtime`.
  #[napi(constructor)]
  pub fn new(env: Env) -> Result<Self> {
    let disposed = Arc::new(AtomicBool::new(false));

    let hook_data = Arc::clone(&disposed);
    env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
      flag.store(true, Ordering::SeqCst);
    })?;

    let qjs_runtime =
      rquickjs::Runtime::new().map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(Self {
      env,
      disposed,
      qjs_runtime,
    })
  }

  /// Get a weak reference to the engine runtime.
  #[napi]
  pub fn weak(&self) -> WeakEngine {
    WeakEngine {
      inner: self.qjs_runtime.weak(),
    }
  }

  /// Whether the owning addon context has been disposed.
  #[napi(js_name = "is_disposed")]
  pub fn is_disposed(&self) -> bool {
    self.disposed.load(Ordering::SeqCst)
  }

  /// `create_array` — build a JS array.
  #[napi(js_name = "create_array")]
  pub fn create_array(&self, len: u32) -> Result<Array<'_>> {
    self.ensure_alive()?;
    let mut raw_val = std::ptr::null_mut();
    napi::check_status!(
      unsafe {
        napi::sys::napi_create_array_with_length(self.env.raw(), len as usize, &mut raw_val)
      },
      "Create array with length failed"
    )?;
    unsafe { Array::from_napi_value(self.env.raw(), raw_val) }
  }

  /// `create_string` — build a JS string.
  #[napi(js_name = "create_string")]
  pub fn create_string(&self, value: String) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.env.create_string(&value)?.into_unknown(&self.env)
  }

  /// `create_int32` — build a JS number from an `i32`.
  #[napi(js_name = "create_int32")]
  pub fn create_int32(&self, int: i32) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.env.create_int32(int)?.into_unknown(&self.env)
  }

  /// `create_double` — build a JS number from an `f64`.
  #[napi(js_name = "create_double")]
  pub fn create_double(&self, value: f64) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.env.create_double(value)?.into_unknown(&self.env)
  }

  /// Does the global object have this property?
  #[napi(js_name = "has_global")]
  pub fn has_global(&self, name: String) -> Result<bool> {
    self.ensure_alive()?;
    let global = self.env.get_global()?;
    global.has_named_property(&name)
  }

  /// `get_napi_version` — report host N-API version.
  #[napi(js_name = "get_napi_version")]
  pub fn get_napi_version(&self) -> Result<u32> {
    self.ensure_alive()?;
    self.env.get_napi_version()
  }

  /// `make_object` — build a JS object.
  #[napi(js_name = "make_object")]
  pub fn make_object(&self) -> Result<Object<'_>> {
    self.ensure_alive()?;
    let mut obj = Object::new(&self.env)?;
    obj.set_named_property("ok", true)?;
    obj.set_named_property("engine", "napi")?;
    Ok(obj)
  }

  /// Persist VM-specific state on the Agent.
  #[napi(js_name = "set_instance_data")]
  pub fn set_instance_data(&self, value: String) -> Result<()> {
    self.ensure_alive()?;
    self.env.set_instance_data(value, (), |_ctx| {})
  }

  /// Read back persisted VM-specific state.
  #[napi(js_name = "get_instance_data")]
  pub fn get_instance_data(&self) -> Result<Option<String>> {
    self.ensure_alive()?;
    Ok(
      self
        .env
        .get_instance_data::<String>()?
        .map(|s: &mut String| s.clone()),
    )
  }

  // --- QuickJS Runtime methods ---

  /// Set the info of the runtime engine.
  #[napi(js_name = "set_info")]
  pub fn set_info(&self, info: String) -> Result<()> {
    self
      .qjs_runtime
      .set_info(info)
      .map_err(|e| Error::from_reason(e.to_string()))
  }

  #[napi(js_name = "setInfo")]
  pub fn set_info_camel(&self, info: String) -> Result<()> {
    self.set_info(info)
  }

  /// Set a limit on the max amount of memory the engine will use.
  #[napi(js_name = "set_memory_limit")]
  pub fn set_memory_limit(&self, limit: i64) {
    self.qjs_runtime.set_memory_limit(limit.max(0) as usize);
  }

  #[napi(js_name = "setMemoryLimit")]
  pub fn set_memory_limit_camel(&self, limit: i64) {
    self.set_memory_limit(limit);
  }

  /// Set a limit on the max size of stack the engine will use.
  #[napi(js_name = "set_max_stack_size")]
  pub fn set_max_stack_size(&self, limit: i64) {
    self.qjs_runtime.set_max_stack_size(limit.max(0) as usize);
  }

  #[napi(js_name = "setMaxStackSize")]
  pub fn set_max_stack_size_camel(&self, limit: i64) {
    self.set_max_stack_size(limit);
  }

  /// Set a memory threshold for garbage collection.
  #[napi(js_name = "set_gc_threshold")]
  pub fn set_gc_threshold(&self, threshold: i64) {
    self.qjs_runtime.set_gc_threshold(threshold.max(0) as usize);
  }

  #[napi(js_name = "setGcThreshold")]
  pub fn set_gc_threshold_camel(&self, threshold: i64) {
    self.set_gc_threshold(threshold);
  }

  /// Set debug flags for dumping memory.
  #[napi(js_name = "set_dump_flags")]
  pub fn set_dump_flags(&self, flags: i64) {
    self.qjs_runtime.set_dump_flags(flags.max(0) as u64);
  }

  #[napi(js_name = "setDumpFlags")]
  pub fn set_dump_flags_camel(&self, flags: i64) {
    self.set_dump_flags(flags);
  }

  /// Manually run garbage collection.
  #[napi(js_name = "run_gc")]
  pub fn run_gc(&self) {
    self.qjs_runtime.run_gc();
  }

  #[napi(js_name = "runGc")]
  pub fn run_gc_camel(&self) {
    self.run_gc();
  }

  /// Get memory usage stats.
  #[napi(js_name = "memory_usage")]
  pub fn memory_usage(&self) -> MemoryUsage {
    let usage = self.qjs_runtime.memory_usage();
    MemoryUsage {
      memory_used_size: usage.memory_used_size as i64,
      malloc_count: usage.malloc_count as i64,
      malloc_size: usage.malloc_size as i64,
      malloc_limit: usage.malloc_limit as i64,
      memory_usage_count: usage.memory_used_count as i64,
    }
  }

  #[napi(js_name = "memoryUsage")]
  pub fn memory_usage_camel(&self) -> MemoryUsage {
    self.memory_usage()
  }

  /// Test for pending jobs in the engine.
  #[napi(js_name = "is_job_pending")]
  pub fn is_job_pending(&self) -> bool {
    self.qjs_runtime.is_job_pending()
  }

  #[napi(js_name = "isJobPending")]
  pub fn is_job_pending_camel(&self) -> bool {
    self.is_job_pending()
  }

  /// Execute first pending job.
  #[napi(js_name = "execute_pending_job")]
  pub fn execute_pending_job(&self) -> Result<bool> {
    self
      .qjs_runtime
      .execute_pending_job()
      .map_err(|e| Error::from_reason(e.to_string()))
  }

  /// `execPendingJob` / `exec_pending_job` alias for executing pending job.
  #[napi(js_name = "exec_pending_job")]
  pub fn exec_pending_job(&self) -> Result<bool> {
    self.execute_pending_job()
  }

  #[napi(js_name = "execPendingJob")]
  pub fn exec_pending_job_camel(&self) -> Result<bool> {
    self.execute_pending_job()
  }

  /// Set module loader with built-in resolver and loader.
  #[napi(js_name = "set_builtin_loader")]
  pub fn set_builtin_loader(&self) {
    self.qjs_runtime.set_loader(
      rquickjs::loader::BuiltinResolver::default(),
      rquickjs::loader::BuiltinLoader::default(),
    );
  }

  #[napi(js_name = "setBuiltinLoader")]
  pub fn set_builtin_loader_camel(&self) {
    self.set_builtin_loader();
  }

  /// Set module loader with file resolver and script loader.
  #[napi(js_name = "set_file_loader")]
  pub fn set_file_loader(&self) {
    self.qjs_runtime.set_loader(
      rquickjs::loader::FileResolver::default(),
      rquickjs::loader::ScriptLoader::default(),
    );
  }

  #[napi(js_name = "setFileLoader")]
  pub fn set_file_loader_camel(&self) {
    self.set_file_loader();
  }

  /// Set module loader by kind names.
  #[napi(js_name = "set_loader")]
  pub fn set_loader(&self, resolver_type: Option<String>, loader_type: Option<String>) {
    let res_kind = resolver_type.as_deref().unwrap_or("file");
    let load_kind = loader_type.as_deref().unwrap_or("script");

    match (res_kind, load_kind) {
      ("builtin", "builtin") => {
        self.qjs_runtime.set_loader(
          rquickjs::loader::BuiltinResolver::default(),
          rquickjs::loader::BuiltinLoader::default(),
        );
      }
      ("builtin", _) => {
        self.qjs_runtime.set_loader(
          rquickjs::loader::BuiltinResolver::default(),
          rquickjs::loader::ScriptLoader::default(),
        );
      }
      _ => {
        self.qjs_runtime.set_loader(
          rquickjs::loader::FileResolver::default(),
          rquickjs::loader::ScriptLoader::default(),
        );
      }
    }
  }

  #[napi(js_name = "setLoader")]
  pub fn set_loader_camel(&self, resolver_type: Option<String>, loader_type: Option<String>) {
    self.set_loader(resolver_type, loader_type);
  }

  fn ensure_alive(&self) -> Result<()> {
    if self.disposed.load(Ordering::SeqCst) {
      return Err(Error::new(
        Status::GenericFailure,
        "Engine: N-API context has been disposed (addon unloaded)",
      ));
    }
    Ok(())
  }
}

impl JsEngine {
  pub fn inner_runtime(&self) -> &rquickjs::Runtime {
    &self.qjs_runtime
  }

  pub fn set_interrupt_handler(&self, handler: Option<Box<dyn FnMut() -> bool + Send>>) {
    self
      .qjs_runtime
      .set_interrupt_handler(handler.map(|h| h as Box<dyn FnMut() -> bool>));
  }
}
