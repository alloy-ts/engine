//! JS/TS bindings for `boa_engine::Context`.
//!
//! See `docs/api/context.md` for details.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use napi::{Env, Error, JsValue as NapiJsValueTrait, sys, Unknown};
use napi::bindgen_prelude::{FromNapiValue, ToNapiValue};
use napi::check_status;
use napi_derive::napi;
use serde_json::Value as JsonValue;

use boa_engine::{
    js_string, Context as BoaContext, JsValue, NativeFunction, Source,
};
use boa_engine::object::ObjectInitializer;
use boa_engine::optimizer::OptimizerOptions;
use boa_engine::property::Attribute;

use crate::realm::{get_realm, insert_realm, Realm};

/// Map a boa [`JsError`](boa_engine::JsError) onto a `napi::Error`.
pub(crate) fn to_napi_err(e: boa_engine::JsError) -> Error {
    Error::from_reason(e.to_string())
}

/// Convert a `napi::Error` into a boa `JsError`.
fn napi_to_js_err(e: Error, _context: &mut BoaContext) -> boa_engine::JsError {
    boa_engine::JsError::from_opaque(js_string!(e.to_string()).into())
}

/// Marshal a boa `JsValue` into a `serde_json::Value` for JS.
pub(crate) fn jsvalue_to_json(value: &JsValue, context: &mut BoaContext) -> napi::Result<JsonValue> {
    Ok(value
        .to_json(context)
        .map_err(to_napi_err)?
        .unwrap_or(JsonValue::Null))
}

/// Convert a JS-provided `serde_json::Value` into a boa `JsValue`.
pub(crate) fn json_to_jsvalue(value: JsonValue, context: &mut BoaContext) -> napi::Result<JsValue> {
    JsValue::from_json(&value, context).map_err(to_napi_err)
}

pub(crate) struct ContextEntry {
    pub(crate) inner: RefCell<BoaContext>,
    pub(crate) host_data: RefCell<HashMap<String, JsValue>>,
    pub(crate) job_queue: RefCell<Vec<(Env, sys::napi_ref)>>,
}

thread_local! {
    static CONTEXTS: RefCell<HashMap<u32, Rc<ContextEntry>>> = RefCell::new(HashMap::new());
}

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

pub(crate) fn alloc_id() -> u32 {
    loop {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        if id == 0 {
            continue;
        }
        let taken = CONTEXTS.with(|c| c.borrow().contains_key(&id));
        if taken {
            continue;
        }
        return id;
    }
}

fn insert_context(ctx: BoaContext) -> u32 {
    let id = alloc_id();
    CONTEXTS.with(|c| {
        c.borrow_mut().insert(
            id,
            Rc::new(ContextEntry {
                inner: RefCell::new(ctx),
                host_data: RefCell::new(HashMap::new()),
                job_queue: RefCell::new(Vec::new()),
            }),
        )
    });
    id
}

pub(crate) fn context_entry(id: u32) -> Option<Rc<ContextEntry>> {
    CONTEXTS.with(|c| c.borrow().get(&id).cloned())
}

fn install_default_globals(ctx: &mut BoaContext) {
    let make = |is_err: bool| {
        unsafe {
            NativeFunction::from_closure(move |_this, args, context| {
                let parts: Vec<String> = args
                    .iter()
                    .filter_map(|a| a.to_string(context).ok())
                    .map(|s| s.to_std_string_escaped())
                    .collect();
                let line = parts.join(" ");
                if is_err {
                    eprintln!("{}", line);
                } else {
                    println!("{}", line);
                }
                Ok(JsValue::undefined())
            })
        }
    };

    let console = ObjectInitializer::new(ctx)
        .function(make(false), js_string!("log"), 0)
        .function(make(false), js_string!("info"), 0)
        .function(make(false), js_string!("debug"), 0)
        .function(make(true), js_string!("error"), 0)
        .function(make(true), js_string!("warn"), 0)
        .function(make(false), js_string!("trace"), 0)
        .build();

    let _ = ctx.register_global_property(js_string!("console"), console, Attribute::all());
}

pub(crate) fn attribute_from_u32(flags: u32) -> Attribute {
    let mut a = Attribute::empty();
    if flags & 1 != 0 {
        a |= Attribute::WRITABLE;
    }
    if flags & 2 != 0 {
        a |= Attribute::ENUMERABLE;
    }
    if flags & 4 != 0 {
        a |= Attribute::CONFIGURABLE;
    }
    a
}

fn jsvalue_to_napi_raw(
    value: &JsValue,
    context: &mut BoaContext,
    env: &Env,
) -> napi::Result<sys::napi_value> {
    if value.is_undefined() {
        let mut val = ptr::null_mut();
        check_status!(unsafe { sys::napi_get_undefined(env.raw(), &mut val) }, "get_undefined failed")?;
        return Ok(val);
    }
    if value.is_null() {
        let mut val = ptr::null_mut();
        check_status!(unsafe { sys::napi_get_null(env.raw(), &mut val) }, "get_null failed")?;
        return Ok(val);
    }
    let json = jsvalue_to_json(value, context)?;
    unsafe { <JsonValue as ToNapiValue>::to_napi_value(env.raw(), json) }
}

#[napi(object)]
pub struct RuntimeLimitsView {
    pub stack_size: Option<u32>,
    pub loop_iteration_limit: Option<i64>,
    pub backtrace_limit: Option<u32>,
    pub recursion_limit: Option<u32>,
}

#[napi(object)]
pub struct OptimizerOptionsView {
    pub statistics: Option<bool>,
    pub constant_folding: Option<bool>,
    pub strength_reduction: Option<bool>,
    pub dead_code_elimination: Option<bool>,
}

#[napi(object)]
pub struct StackFrameView {
    pub function_name: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[napi]
pub struct Context {
    id: u32,
}

#[napi]
impl Context {
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        let mut ctx = BoaContext::builder().build().map_err(to_napi_err)?;
        install_default_globals(&mut ctx);
        Ok(Context { id: insert_context(ctx) })
    }

    #[napi]
    pub fn builder() -> ContextBuilder {
        ContextBuilder { can_block: false }
    }

    #[napi]
    pub fn dispose(&self) -> napi::Result<()> {
        CONTEXTS.with(|c| c.borrow_mut().remove(&self.id));
        Ok(())
    }

    fn entry(&self) -> napi::Result<Rc<ContextEntry>> {
        CONTEXTS
            .with(|c| c.borrow().get(&self.id).cloned())
            .ok_or_else(|| Error::from_reason("Context has been disposed"))
    }

    #[napi]
    pub fn eval(&self, source: String) -> napi::Result<JsonValue> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let value = ctx
            .eval(Source::from_bytes(source.as_bytes()))
            .map_err(to_napi_err)?;
        jsvalue_to_json(&value, &mut ctx)
    }

    #[napi]
    pub fn run_jobs(&self) -> napi::Result<()> {
        let entry = self.entry()?;
        {
            let mut ctx = entry.inner.borrow_mut();
            ctx.run_jobs().map_err(to_napi_err)?;
        }
        let jobs: Vec<(Env, sys::napi_ref)> = entry.job_queue.borrow_mut().drain(..).collect();
        for (env, cb_ref) in jobs {
            let mut cb_val = ptr::null_mut();
            check_status!(
                unsafe { sys::napi_get_reference_value(env.raw(), cb_ref, &mut cb_val) },
                "Failed to get reference for enqueued job"
            )?;
            let global = env.get_global()?;
            let global_raw = NapiJsValueTrait::raw(&global);
            let mut return_value = ptr::null_mut();
            let call_res = unsafe {
                sys::napi_call_function(
                    env.raw(),
                    global_raw,
                    cb_val,
                    0,
                    ptr::null_mut(),
                    &mut return_value,
                )
            };
            unsafe {
                let _ = sys::napi_delete_reference(env.raw(), cb_ref);
            }
            check_status!(call_res, "enqueued job threw")?;
        }
        Ok(())
    }

    #[napi]
    pub fn enqueue_job(&self, env: Env, callback: Unknown) -> napi::Result<()> {
        let entry = self.entry()?;
        let mut cb_ref = ptr::null_mut();
        let raw = NapiJsValueTrait::raw(&callback);
        check_status!(
            unsafe { sys::napi_create_reference(env.raw(), raw, 1, &mut cb_ref) },
            "Failed to create reference for callback"
        )?;
        entry.job_queue.borrow_mut().push((env, cb_ref));
        Ok(())
    }

    #[napi]
    pub fn strict(&self, mode: bool) -> napi::Result<()> {
        let entry = self.entry()?;
        entry.inner.borrow_mut().strict(mode);
        Ok(())
    }

    #[napi]
    pub fn register_global_property(
        &self,
        key: String,
        value: JsonValue,
        attribute: u32,
    ) -> napi::Result<()> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let value = json_to_jsvalue(value, &mut ctx)?;
        ctx.register_global_property(js_string!(key.as_str()), value, attribute_from_u32(attribute))
            .map_err(to_napi_err)
    }

    #[napi]
    pub fn register_global_callable(
        &self,
        env: Env,
        name: String,
        length: u32,
        callback: Unknown,
    ) -> napi::Result<()> {
        self.register_callable(env, name, length, callback, true)
    }

    #[napi]
    pub fn register_global_builtin_callable(
        &self,
        env: Env,
        name: String,
        length: u32,
        callback: Unknown,
    ) -> napi::Result<()> {
        self.register_callable(env, name, length, callback, false)
    }

    fn register_callable(
        &self,
        env: Env,
        name: String,
        length: u32,
        callback: Unknown,
        constructable: bool,
    ) -> napi::Result<()> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let mut cb_ref = ptr::null_mut();
        let raw = NapiJsValueTrait::raw(&callback);
        check_status!(
            unsafe { sys::napi_create_reference(env.raw(), raw, 1, &mut cb_ref) },
            "Failed to create reference for callable callback"
        )?;

        let native = unsafe {
            NativeFunction::from_closure(move |this, args, context| {
                let mut cb_val = ptr::null_mut();
                if sys::napi_get_reference_value(env.raw(), cb_ref, &mut cb_val) != sys::Status::napi_ok {
                    return Err(napi_to_js_err(
                        Error::from_reason("Failed to retrieve callable reference"),
                        context,
                    ));
                }
                let this_n = jsvalue_to_napi_raw(this, context, &env)
                    .map_err(|e| napi_to_js_err(e, context))?;
                let mut argv: Vec<sys::napi_value> = Vec::with_capacity(args.len());
                for a in args {
                    argv.push(
                        jsvalue_to_napi_raw(a, context, &env)
                            .map_err(|e| napi_to_js_err(e, context))?,
                    );
                }
                let mut return_value = ptr::null_mut();
                let status = sys::napi_call_function(
                    env.raw(),
                    this_n,
                    cb_val,
                    argv.len(),
                    argv.as_ptr(),
                    &mut return_value,
                );
                if status != sys::Status::napi_ok {
                    return Err(napi_to_js_err(
                        Error::from_reason("registered global callable threw"),
                        context,
                    ));
                }
                let json = match <JsonValue as FromNapiValue>::from_napi_value(env.raw(), return_value) {
                    Ok(v) => v,
                    Err(e) => return Err(napi_to_js_err(e, context)),
                };
                JsValue::from_json(&json, context)
            })
        };
        if constructable {
            ctx.register_global_callable(js_string!(name.as_str()), length as usize, native)
                .map_err(to_napi_err)
        } else {
            ctx.register_global_builtin_callable(js_string!(name.as_str()), length as usize, native)
                .map_err(to_napi_err)
        }
    }

    #[napi]
    pub fn global_keys(&self) -> napi::Result<Vec<String>> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let v = ctx
            .eval(Source::from_bytes(b"Object.keys(globalThis)"))
            .map_err(to_napi_err)?;
        match jsvalue_to_json(&v, &mut ctx)? {
            JsonValue::Array(arr) => Ok(arr
                .into_iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()),
            _ => Ok(Vec::new()),
        }
    }

    #[napi]
    pub fn global_object(&self) -> napi::Result<JsonValue> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let code = r#"
            (() => {
                const res = {};
                for (const key of Object.keys(globalThis)) {
                    if (key !== 'globalThis' && key !== 'window' && key !== 'self' && key !== 'top') {
                        res[key] = globalThis[key];
                    }
                }
                return res;
            })()
        "#;
        let v = ctx.eval(Source::from_bytes(code.as_bytes())).map_err(to_napi_err)?;
        jsvalue_to_json(&v, &mut ctx)
    }

    #[napi]
    pub fn can_block(&self) -> napi::Result<bool> {
        let entry = self.entry()?;
        let r = entry.inner.borrow().can_block();
        Ok(r)
    }

    #[napi]
    pub fn runtime_limits(&self) -> napi::Result<RuntimeLimitsView> {
        let entry = self.entry()?;
        let rl = entry.inner.borrow().runtime_limits();
        Ok(RuntimeLimitsView {
            stack_size: Some(rl.stack_size_limit() as u32),
            loop_iteration_limit: Some(rl.loop_iteration_limit() as i64),
            backtrace_limit: Some(rl.backtrace_limit() as u32),
            recursion_limit: Some(rl.recursion_limit() as u32),
        })
    }

    #[napi]
    pub fn set_runtime_limits(&self, limits: RuntimeLimitsView) -> napi::Result<()> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let mut rl = ctx.runtime_limits().clone();
        if let Some(v) = limits.stack_size {
            rl.set_stack_size_limit(v as usize);
        }
        if let Some(v) = limits.loop_iteration_limit {
            rl.set_loop_iteration_limit(v as u64);
        }
        if let Some(v) = limits.backtrace_limit {
            rl.set_backtrace_limit(v as usize);
        }
        if let Some(v) = limits.recursion_limit {
            rl.set_recursion_limit(v as usize);
        }
        ctx.set_runtime_limits(rl);
        Ok(())
    }

    #[napi]
    pub fn optimizer_options(&self) -> napi::Result<OptimizerOptionsView> {
        let entry = self.entry()?;
        let o = entry.inner.borrow().optimizer_options();
        Ok(OptimizerOptionsView {
            statistics: Some(o.contains(OptimizerOptions::STATISTICS)),
            constant_folding: Some(o.contains(OptimizerOptions::CONSTANT_FOLDING)),
            strength_reduction: Some(o.contains(OptimizerOptions::STRENGTH_REDUCTION)),
            dead_code_elimination: Some(o.contains(OptimizerOptions::DEAD_CODE_ELIMINATION)),
        })
    }

    #[napi]
    pub fn set_optimizer_options(&self, options: OptimizerOptionsView) -> napi::Result<()> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let mut o = OptimizerOptions::empty();
        if options.statistics.unwrap_or(false) {
            o |= OptimizerOptions::STATISTICS;
        }
        if options.constant_folding.unwrap_or(false) {
            o |= OptimizerOptions::CONSTANT_FOLDING;
        }
        if options.strength_reduction.unwrap_or(false) {
            o |= OptimizerOptions::STRENGTH_REDUCTION;
        }
        if options.dead_code_elimination.unwrap_or(false) {
            o |= OptimizerOptions::DEAD_CODE_ELIMINATION;
        }
        ctx.set_optimizer_options(o);
        Ok(())
    }

    #[napi]
    pub fn stack_trace(&self) -> napi::Result<Vec<StackFrameView>> {
        let entry = self.entry()?;
        let ctx = entry.inner.borrow();
        let frames: Vec<StackFrameView> = ctx
            .stack_trace()
            .map(|f| {
                let loc = f.position();
                StackFrameView {
                    function_name: loc.function_name.to_std_string_escaped(),
                    line: loc.position.map(|p| p.line_number()),
                    column: loc.position.map(|p| p.column_number()),
                }
            })
            .collect();
        Ok(frames)
    }

    #[napi]
    pub fn create_realm(&self) -> napi::Result<Realm> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let realm = ctx.create_realm().map_err(to_napi_err)?;
        Ok(Realm {
            id: insert_realm(realm),
            ctx_id: self.id,
        })
    }

    #[napi]
    pub fn enter_realm(&self, handle: Realm) -> napi::Result<Realm> {
        let entry = self.entry()?;
        let realm = get_realm(handle.id)
            .ok_or_else(|| Error::from_reason("invalid Realm handle"))?;
        let mut ctx = entry.inner.borrow_mut();
        let prev = ctx.enter_realm(realm);
        Ok(Realm {
            id: insert_realm(prev),
            ctx_id: self.id,
        })
    }

    #[napi]
    pub fn realm(&self) -> napi::Result<Realm> {
        let entry = self.entry()?;
        let realm = entry.inner.borrow().realm().clone();
        Ok(Realm {
            id: insert_realm(realm),
            ctx_id: self.id,
        })
    }

    #[napi]
    pub fn insert_data(&self, key: String, value: JsonValue) -> napi::Result<()> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let v = json_to_jsvalue(value, &mut ctx)?;
        entry.host_data.borrow_mut().insert(key, v);
        Ok(())
    }

    #[napi]
    pub fn has_data(&self, key: String) -> napi::Result<bool> {
        let entry = self.entry()?;
        let r = entry.host_data.borrow().contains_key(&key);
        Ok(r)
    }

    #[napi]
    pub fn get_data(&self, key: String) -> napi::Result<JsonValue> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let v = entry.host_data.borrow().get(&key).cloned();
        match v {
            Some(v) => jsvalue_to_json(&v, &mut ctx),
            None => Ok(JsonValue::Null),
        }
    }

    #[napi]
    pub fn remove_data(&self, key: String) -> napi::Result<JsonValue> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let v = entry.host_data.borrow_mut().remove(&key);
        match v {
            Some(v) => jsvalue_to_json(&v, &mut ctx),
            None => Ok(JsonValue::Null),
        }
    }

    #[napi]
    pub fn clear_kept_objects(&self) -> napi::Result<()> {
        let entry = self.entry()?;
        entry.inner.borrow_mut().clear_kept_objects();
        Ok(())
    }
}

#[napi]
pub struct ContextBuilder {
    can_block: bool,
}

#[napi]
impl ContextBuilder {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self { can_block: false }
    }

    #[napi]
    pub fn can_block(&mut self, can: bool) {
        self.can_block = can;
    }

    #[napi]
    pub fn build(&self) -> napi::Result<Context> {
        let mut ctx = BoaContext::builder()
            .can_block(self.can_block)
            .build()
            .map_err(to_napi_err)?;
        install_default_globals(&mut ctx);
        Ok(Context { id: insert_context(ctx) })
    }
}
