//! NAPI bindings for QuickJS [`Context`].

use napi::bindgen_prelude::*;
use napi::Error;
use napi_derive::napi;

use rquickjs::context::EvalOptions as QuickJsEvalOptions;

use crate::runtime::JsRuntime;

/// Convert a QuickJS `Value` into a `serde_json::Value`.
pub(crate) fn js_value_to_serde<'js>(
  ctx: &rquickjs::Ctx<'js>,
  val: rquickjs::Value<'js>,
) -> Result<serde_json::Value> {
  if val.is_undefined() || val.is_null() {
    return Ok(serde_json::Value::Null);
  }
  if let Some(b) = val.as_bool() {
    return Ok(serde_json::Value::Bool(b));
  }
  if let Some(i) = val.as_int() {
    return Ok(serde_json::Value::Number(i.into()));
  }
  if let Some(f) = val.as_float() {
    if let Some(num) = serde_json::Number::from_f64(f) {
      return Ok(serde_json::Value::Number(num));
    }
    return Ok(serde_json::Value::Null);
  }
  if let Some(s) = val.as_string() {
    if let Ok(str_val) = s.to_string() {
      return Ok(serde_json::Value::String(str_val));
    }
  }
  if let Ok(Some(json_str)) = ctx.json_stringify(val) {
    if let Ok(str_val) = json_str.to_string() {
      if let Ok(json_val) = serde_json::from_str(&str_val) {
        return Ok(json_val);
      }
    }
  }
  Ok(serde_json::Value::Null)
}

/// Evaluation options for scripts.
#[napi(object)]
#[derive(Default, Debug, Clone)]
pub struct EvalOptions {
  pub global: Option<bool>,
  pub strict: Option<bool>,
  pub backtrace_barrier: Option<bool>,
  pub promise: Option<bool>,
  pub filename: Option<String>,
}

fn to_rquickjs_eval_options(opts: Option<EvalOptions>) -> QuickJsEvalOptions {
  let mut res = QuickJsEvalOptions::default();
  if let Some(o) = opts {
    if let Some(v) = o.global {
      res.global = v;
    }
    if let Some(v) = o.strict {
      res.strict = v;
    }
    if let Some(v) = o.backtrace_barrier {
      res.backtrace_barrier = v;
    }
    if let Some(v) = o.promise {
      res.promise = v;
    }
    if let Some(v) = o.filename {
      res.filename = Some(v);
    }
  }
  res
}

/// NAPI wrapper for QuickJS execution [`Context`].
#[napi(js_name = "Context")]
#[derive(Clone)]
pub struct JsContext {
  pub(crate) inner: rquickjs::Context,
}

#[napi]
impl JsContext {
  /// Create a full context associated with the given runtime (or a new runtime if omitted).
  #[napi(constructor)]
  pub fn new(runtime: Option<&JsRuntime>) -> Result<Self> {
    Self::full(runtime)
  }

  /// Create a base context with minimum registered intrinsics.
  #[napi(factory, js_name = "base")]
  pub fn base(runtime: Option<&JsRuntime>) -> Result<Self> {
    let ctx = if let Some(rt) = runtime {
      rquickjs::Context::base(&rt.inner)
    } else {
      let rt = rquickjs::Runtime::new().map_err(|e| Error::from_reason(e.to_string()))?;
      rquickjs::Context::base(&rt)
    }
    .map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(Self { inner: ctx })
  }

  /// Create a context with all standard intrinsics registered.
  #[napi(factory, js_name = "full")]
  pub fn full(runtime: Option<&JsRuntime>) -> Result<Self> {
    let ctx = if let Some(rt) = runtime {
      rquickjs::Context::full(&rt.inner)
    } else {
      let rt = rquickjs::Runtime::new().map_err(|e| Error::from_reason(e.to_string()))?;
      rquickjs::Context::full(&rt)
    }
    .map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(Self { inner: ctx })
  }

  /// Evaluate a JavaScript script in global context.
  #[napi]
  pub fn eval(&self, source: String, options: Option<EvalOptions>) -> Result<serde_json::Value> {
    let eval_opts = to_rquickjs_eval_options(options);
    self.inner.with(|ctx| {
      let res: rquickjs::Result<rquickjs::Value> = ctx.eval_with_options(source, eval_opts);
      match res {
        Ok(val) => js_value_to_serde(&ctx, val),
        Err(e) => {
          if ctx.has_exception() {
            let catch_val = ctx.catch();
            let err_str = catch_val
              .as_string()
              .and_then(|s| s.to_string().ok())
              .unwrap_or_else(|| format!("{:?}", catch_val));
            Err(Error::from_reason(err_str))
          } else {
            Err(Error::from_reason(e.to_string()))
          }
        }
      }
    })
  }

  /// Evaluate a script with top level await / promise support, driving pending jobs to completion.
  #[napi(js_name = "eval_promise")]
  pub fn eval_promise(&self, source: String) -> Result<serde_json::Value> {
    self.inner.with(|ctx| {
      let mut opts = QuickJsEvalOptions::default();
      opts.promise = true;
      let res: rquickjs::Result<rquickjs::Value> = ctx.eval_with_options(source, opts);
      match res {
        Ok(val) => {
          while ctx.execute_pending_job() {}
          if let Ok(p) = rquickjs::Promise::from_value(val.clone()) {
            let res_val: rquickjs::Result<rquickjs::Value> = p.finish();
            match res_val {
              Ok(v) => {
                if let Some(obj) = v.as_object() {
                  if let Ok(inner_val) = obj.get::<_, rquickjs::Value>("value") {
                    return js_value_to_serde(&ctx, inner_val);
                  }
                }
                js_value_to_serde(&ctx, v)
              }
              Err(e) => {
                if ctx.has_exception() {
                  let catch_val = ctx.catch();
                  let err_str = catch_val
                    .as_string()
                    .and_then(|s| s.to_string().ok())
                    .unwrap_or_else(|| format!("{:?}", catch_val));
                  Err(Error::from_reason(err_str))
                } else {
                  Err(Error::from_reason(e.to_string()))
                }
              }
            }
          } else {
            js_value_to_serde(&ctx, val)
          }
        }
        Err(e) => {
          if ctx.has_exception() {
            let catch_val = ctx.catch();
            let err_str = catch_val
              .as_string()
              .and_then(|s| s.to_string().ok())
              .unwrap_or_else(|| format!("{:?}", catch_val));
            Err(Error::from_reason(err_str))
          } else {
            Err(Error::from_reason(e.to_string()))
          }
        }
      }
    })
  }

  /// Compiles a script without evaluating it.
  #[napi]
  pub fn compile(&self, source: String, filename: String, strict: Option<bool>) -> Result<()> {
    let is_strict = strict.unwrap_or(true);
    let c_filename =
      std::ffi::CString::new(filename).map_err(|e| Error::from_reason(e.to_string()))?;
    let c_source =
      std::ffi::CString::new(source.as_str()).map_err(|e| Error::from_reason(e.to_string()))?;
    let len = source.len();

    self.inner.with(|ctx| {
      let mut flags =
        rquickjs::qjs::JS_EVAL_TYPE_GLOBAL | rquickjs::qjs::JS_EVAL_FLAG_COMPILE_ONLY;
      if is_strict {
        flags |= rquickjs::qjs::JS_EVAL_FLAG_STRICT;
      }
      unsafe {
        let val = rquickjs::qjs::JS_Eval(
          ctx.as_raw().as_ptr(),
          c_source.as_ptr(),
          len as _,
          c_filename.as_ptr(),
          flags as i32,
        );
        if rquickjs::qjs::JS_IsException(val) {
          if ctx.has_exception() {
            let catch_val = ctx.catch();
            let err_str = catch_val
              .as_string()
              .and_then(|s| s.to_string().ok())
              .unwrap_or_else(|| format!("{:?}", catch_val));
            Err(Error::from_reason(err_str))
          } else {
            Err(Error::from_reason("Compilation failed".to_string()))
          }
        } else {
          rquickjs::qjs::JS_FreeValue(ctx.as_raw().as_ptr(), val);
          Ok(())
        }
      }
    })
  }

  /// Parse JSON string into a JavaScript value representation.
  #[napi(js_name = "json_parse")]
  pub fn json_parse(&self, json_str: String) -> Result<serde_json::Value> {
    self.inner.with(|ctx| {
      let val = ctx.json_parse(json_str).map_err(|e| Error::from_reason(e.to_string()))?;
      js_value_to_serde(&ctx, val)
    })
  }

  /// Stringify a JSON value.
  #[napi(js_name = "json_stringify")]
  pub fn json_stringify(&self, value: serde_json::Value) -> Result<Option<String>> {
    serde_json::to_string(&value)
      .map(Some)
      .map_err(|e| Error::from_reason(e.to_string()))
  }

  /// Execute a pending job in this context's runtime.
  #[napi(js_name = "execute_pending_job")]
  pub fn execute_pending_job(&self) -> bool {
    self.inner.with(|ctx| ctx.execute_pending_job())
  }

  /// Trigger cyclic garbage collection on the context's runtime.
  #[napi(js_name = "run_gc")]
  pub fn run_gc(&self) {
    self.inner.with(|ctx| ctx.run_gc())
  }

  /// Returns the associated runtime wrapper.
  #[napi(js_name = "get_runtime")]
  pub fn get_runtime(&self) -> JsRuntime {
    JsRuntime {
      inner: self.inner.runtime().clone(),
    }
  }
}
