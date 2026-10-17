use std::cell::RefCell;
use std::path::{Path, PathBuf};

use napi::bindgen_prelude::Error;
use napi_derive::napi;
use serde_json::Value as JsonValue;

use boa_engine::{Context, JsValue, Script as BoaScript, Source};
use boa_engine::object::ObjectInitializer;
use boa_engine::native_function::NativeFunction;
use boa_engine::property::Attribute;
use boa_engine::js_string;

fn install_default_globals(ctx: &mut Context) {
  let make = |is_err: bool| {
    NativeFunction::from_copy_closure(move |_this, args, context| {
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

/// Map a Boa [`JsError`](boa_engine::JsError) onto a `napi::Error` so the JS
/// caller sees a thrown exception.
fn to_napi_err(e: boa_engine::JsError) -> Error {
  Error::from_reason(e.to_string())
}

/// Convert a Boa [`JsValue`] into a JSON value Node can receive directly.
///
/// `undefined` is not JSON-representable and surfaces as `null`, matching the
/// convention used by [`crate::context::value_to_json`].
fn jsvalue_to_json(value: &JsValue, context: &mut Context) -> napi::Result<JsonValue> {
  Ok(
    value
      .to_json(context)
      .map_err(to_napi_err)?
      .unwrap_or(JsonValue::Null),
  )
}

/// An ECMAScript script, parsed and evaluated by [Boa](https://github.com/boa-dev/boa).
///
/// Each instance owns a private Boa `Context` — and therefore its own realm and
/// global object — so scripts cannot interfere with one another.
///
/// JS:
/// ```ts
/// const script = new Script("1 + 1");
/// script.evaluate(); // 2
/// ```
#[napi]
pub struct Script {
  /// The parsed script record. Declared first so it is dropped *before*
  /// `context` releases Boa's GC heap (see the module docs).
  script: BoaScript,
  /// The context the script was parsed in. Behind a `RefCell` because Boa's
  /// entry points need `&mut Context` while N-API only gives us `&self`.
  /// Access is serialized by the single-threaded JS environment, which is also
  /// all Boa allows.
  context: RefCell<Context>,
  /// The original source text, kept for the `source` getter.
  source: String,
  /// Optional path naming this script in stack traces and error messages.
  path: Option<PathBuf>,
}

#[napi]
impl Script {
  /// Parse `source` as an ECMAScript script.
  ///
  /// `path` is optional; when given, it names the script in stack traces and
  /// parse errors. Throws (as a JS exception) if `source` fails to parse.
  #[napi(constructor)]
  pub fn new(source: String, path: Option<String>) -> napi::Result<Self> {
    let mut context = Context::default();
    install_default_globals(&mut context);

    let src = match &path {
      Some(p) => Source::from_reader(source.as_bytes(), Some(Path::new(p))),
      None => Source::from_reader(source.as_bytes(), None),
    };

    // `None` realm: parse into the context's default realm.
    let script = BoaScript::parse(src, None, &mut context).map_err(to_napi_err)?;

    Ok(Script {
      script,
      context: RefCell::new(context),
      source,
      path: path.map(PathBuf::from),
    })
  }

  /// Read `path` from disk and parse it as an ECMAScript script.
  ///
  /// Equivalent to `new Script(fs.readFileSync(path, "utf8"), path)`, except
  /// the file is read by Rust. Throws if the file cannot be read or does not
  /// parse.
  #[napi(factory)]
  pub fn from_file(path: String) -> napi::Result<Self> {
    let mut context = Context::default();
    install_default_globals(&mut context);

    let src =
      Source::from_filepath(Path::new(&path)).map_err(|e| Error::from_reason(e.to_string()))?;

    let script = BoaScript::parse(src, None, &mut context).map_err(to_napi_err)?;

    Ok(Script {
      script,
      context: RefCell::new(context),
      // The file contents live in the parser; `source` only tracks the path.
      source: String::new(),
      path: Some(PathBuf::from(path)),
    })
  }

  /// Evaluate the script and return its completion value as a JS value.
  ///
  /// Pending promise jobs are drained after evaluation (see
  /// [`Script::runJobs`]), so microtasks scheduled by the script have settled
  /// by the time this returns.
  ///
  /// Throws (as a JS exception) if the script throws.
  #[napi]
  pub fn evaluate(&self) -> napi::Result<JsonValue> {
    let mut context = self.context.borrow_mut();
    let value = self.script.evaluate(&mut context).map_err(to_napi_err)?;
    context.run_jobs().map_err(to_napi_err)?;
    jsvalue_to_json(&value, &mut context)
  }

  /// Run all pending promise jobs (microtasks) to completion.
  ///
  /// [`Script::evaluate`] already does this; call it only when you need finer
  /// control over when the queue is drained.
  #[napi]
  pub fn run_jobs(&self) -> napi::Result<()> {
    self.context.borrow_mut().run_jobs().map_err(to_napi_err)
  }

  /// The source text this script was parsed from.
  ///
  /// Empty for scripts created with [`Script.fromFile`], which reads the file
  /// straight into Boa's parser.
  #[napi(getter)]
  pub fn source(&self) -> String {
    self.source.clone()
  }

  /// The path of this script, or `null` when it was parsed from a string
  /// without a path.
  #[napi(getter)]
  pub fn path(&self) -> Option<String> {
    self.path.as_ref().map(|p| p.to_string_lossy().into_owned())
  }
}
