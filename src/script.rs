use std::cell::RefCell;
use std::path::{Path, PathBuf};

use napi::bindgen_prelude::Error;
use napi_derive::napi;
use serde_json::Value as JsonValue;

use boa_engine::{Context, JsValue, Script as BoaScript, Source};

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
