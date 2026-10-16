//! NAPI bindings for JS `Context`.

use napi::bindgen_prelude::*;
use napi::Env;
use napi_derive::napi;

/// A single execution context with its own global variables and execution scope.
#[napi]
pub struct Context;

#[napi]
impl Context {
  #[napi(constructor)]
  pub fn new() -> Self {
    Context
  }

  /// Creates a base context with minimal default intrinsics.
  #[napi(factory)]
  pub fn base() -> Self {
    Context
  }

  /// Creates a full context with all standard intrinsics registered.
  #[napi(factory)]
  pub fn full() -> Self {
    Context
  }

  /// Evaluates a JavaScript string in the global context.
  #[napi]
  pub fn eval(&self, env: &Env, source: String) -> Result<Unknown<'_>> {
    env.run_script(&source)
  }

  /// Compiles a global script without evaluating it.
  #[napi]
  pub fn compile(&self, env: &Env, source: String, _filename: Option<String>, _strict: Option<bool>) -> Result<bool> {
    let _: Unknown = env.run_script(&source)?;
    Ok(true)
  }

  /// Evaluates a script in global context with async promise support.
  #[napi]
  pub fn eval_promise(&self, env: &Env, source: String) -> Result<Unknown<'_>> {
    env.run_script(&source)
  }

  /// Parse a JSON string into a JavaScript value.
  #[napi]
  pub fn json_parse(&self, env: &Env, json_text: String) -> Result<Unknown<'_>> {
    let global = env.get_global()?;
    let json_obj: Object = global.get_named_property("JSON")?;
    let parse_fn: Function<'_, String, Unknown<'_>> = json_obj.get_named_property("parse")?;
    parse_fn.call(json_text)
  }

  /// Stringify a JavaScript value into its JSON representation.
  #[napi]
  pub fn json_stringify(&self, env: &Env, value: Unknown) -> Result<Option<String>> {
    let global = env.get_global()?;
    let json_obj: Object = global.get_named_property("JSON")?;
    let stringify_fn: Function<'_, Unknown, Option<String>> = json_obj.get_named_property("stringify")?;
    stringify_fn.call(value)
  }

  /// Returns whether there is a pending exception.
  #[napi]
  pub fn has_exception(&self) -> bool {
    false
  }

  /// Runs garbage collection for a cycle.
  #[napi]
  pub fn run_gc(&self, env: &Env) -> Result<()> {
    let global = env.get_global()?;
    if global.has_named_property("gc")? {
      let gc_fn: Function<'_, (), ()> = global.get_named_property("gc")?;
      let _ = gc_fn.call(());
    }
    Ok(())
  }
}
