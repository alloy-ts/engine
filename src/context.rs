//! NAPI bindings for `rquickjs::Context`.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::{context::EvalOptions as QjsEvalOptions, Context as QjsContext, Runtime as QjsRuntime};
use serde_json::Value as JsonValue;

/// Options for JavaScript evaluation in QuickJS.
#[napi(object)]
pub struct EvalOptions {
  pub global: Option<bool>,
  pub strict: Option<bool>,
  pub backtrace_barrier: Option<bool>,
  pub promise: Option<bool>,
  pub filename: Option<String>,
}

fn to_qjs_eval_options(options: Option<EvalOptions>) -> QjsEvalOptions {
  let mut opts = QjsEvalOptions::default();
  if let Some(o) = options {
    if let Some(v) = o.global {
      opts.global = v;
    }
    if let Some(v) = o.strict {
      opts.strict = v;
    }
    if let Some(v) = o.backtrace_barrier {
      opts.backtrace_barrier = v;
    }
    if let Some(v) = o.promise {
      opts.promise = v;
    }
    opts.filename = o.filename;
  }
  opts
}

/// Helper to convert a QuickJS value to a `serde_json::Value`.
fn qjs_val_to_json<'js>(ctx: &rquickjs::Ctx<'js>, val: rquickjs::Value<'js>) -> Result<JsonValue> {
  if val.is_undefined() || val.is_null() {
    return Ok(JsonValue::Null);
  }
  if val.is_bool() {
    return Ok(JsonValue::Bool(val.as_bool().unwrap_or(false)));
  }
  if val.is_int() {
    return Ok(JsonValue::from(val.as_int().unwrap_or(0)));
  }
  if val.is_float() {
    if let Some(f) = val.as_float() {
      if let Some(num) = serde_json::Number::from_f64(f) {
        return Ok(JsonValue::Number(num));
      }
    }
    return Ok(JsonValue::Null);
  }
  if val.is_string() {
    if let Some(s) = val.as_string() {
      if let Ok(std_s) = s.to_string() {
        return Ok(JsonValue::String(std_s));
      }
    }
  }

  // Fallback to json_stringify for arrays, objects, etc.
  match ctx.json_stringify(val) {
    Ok(Some(s)) => {
      let str_val = s.to_string().map_err(|e| Error::from_reason(e.to_string()))?;
      serde_json::from_str(&str_val).map_err(|e| Error::from_reason(e.to_string()))
    }
    Ok(None) => Ok(JsonValue::Null),
    Err(e) => Err(handle_qjs_error(ctx, e)),
  }
}

/// Format QuickJS exceptions into readable NAPI errors.
fn handle_qjs_error<'js>(ctx: &rquickjs::Ctx<'js>, err: rquickjs::Error) -> Error {
  if ctx.has_exception() {
    let exc = ctx.catch();
    if let Some(obj) = exc.as_object() {
      if let Ok(msg) = obj.get::<_, String>("message") {
        if let Ok(stack) = obj.get::<_, String>("stack") {
          return Error::from_reason(format!("JavaScript Exception: {}\n{}", msg, stack));
        }
        return Error::from_reason(format!("JavaScript Exception: {}", msg));
      }
    }
    if let Some(s) = exc.as_string() {
      if let Ok(std_s) = s.to_string() {
        return Error::from_reason(format!("JavaScript Exception: {}", std_s));
      }
    }
    if let Ok(Some(json_s)) = ctx.json_stringify(exc) {
      if let Ok(std_s) = json_s.to_string() {
        return Error::from_reason(format!("JavaScript Exception: {}", std_s));
      }
    }
  }
  Error::from_reason(err.to_string())
}

/// NAPI binding around [`rquickjs::Context`].
#[napi(js_name = "Context")]
pub struct JsContext {
  pub(crate) inner: QjsContext,
}

#[napi]
impl JsContext {
  /// Create a new `Context` with full standard intrinsics registered.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let runtime = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    let inner = QjsContext::full(&runtime).map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self { inner })
  }

  /// Create a base context with only required intrinsics registered.
  #[napi(factory, js_name = "create_base")]
  pub fn create_base() -> Result<Self> {
    let runtime = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    let inner = QjsContext::base(&runtime).map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self { inner })
  }

  /// Create a context with all standard intrinsics registered.
  #[napi(factory, js_name = "create_full")]
  pub fn create_full() -> Result<Self> {
    Self::new()
  }

  /// Evaluate JavaScript source code in this context.
  #[napi]
  pub fn eval(&self, source: String, options: Option<EvalOptions>) -> Result<JsonValue> {
    let opts = to_qjs_eval_options(options);
    self.inner.with(|ctx| {
      match ctx.eval_with_options::<rquickjs::Value, _>(source.as_bytes(), opts) {
        Ok(val) => qjs_val_to_json(&ctx, val),
        Err(err) => Err(handle_qjs_error(&ctx, err)),
      }
    })
  }

  /// Parse a JSON string into a JS value within this context.
  #[napi(js_name = "json_parse")]
  pub fn json_parse(&self, json_str: String) -> Result<JsonValue> {
    self.inner.with(|ctx| {
      match ctx.json_parse(json_str.as_bytes()) {
        Ok(val) => qjs_val_to_json(&ctx, val),
        Err(err) => Err(handle_qjs_error(&ctx, err)),
      }
    })
  }

  /// Stringify a JSON-serializable value using QuickJS `JSON.stringify`.
  #[napi(js_name = "json_stringify")]
  pub fn json_stringify(&self, value: JsonValue) -> Result<Option<String>> {
    let json_bytes = value.to_string();
    self.inner.with(|ctx| {
      match ctx.json_parse(json_bytes.as_bytes()) {
        Ok(val) => match ctx.json_stringify(val) {
          Ok(Some(s)) => Ok(Some(s.to_string().map_err(|e| Error::from_reason(e.to_string()))?)),
          Ok(None) => Ok(None),
          Err(err) => Err(handle_qjs_error(&ctx, err)),
        },
        Err(err) => Err(handle_qjs_error(&ctx, err)),
      }
    })
  }

  /// Execute a pending QuickJS job (e.g. microtask / promise resolution).
  ///
  /// Returns `true` if a job was executed, or `false` if no jobs were pending.
  #[napi(js_name = "execute_pending_job")]
  pub fn execute_pending_job(&self) -> bool {
    self.inner.with(|ctx| ctx.execute_pending_job())
  }

  /// Run the QuickJS garbage collector for a cycle.
  #[napi(js_name = "run_gc")]
  pub fn run_gc(&self) {
    self.inner.with(|ctx| ctx.run_gc())
  }
}
