use std::path::Path;

use napi::bindgen_prelude::Result as NapiResult;
use napi::Error as NapiError;
use napi_derive::napi;

use oxc::Compiler as OxcCompiler;
use oxc::diagnostics::Diagnostics;
use oxc::span::SourceType;
use oxc_napi::{get_source_type, OxcError};

/// Filename reported in codeframes and used for source-type inference when the
/// caller supplies none.
const DEFAULT_FILENAME: &str = "unnamed";

/// Options for [`Compiler::execute`].
///
/// Mirrors the inference oxc performs for its own N-API package: `filename`'s
/// extension decides the language, and `lang` / `source_type` override it.
#[napi(object)]
pub struct CompilerOptions {
  /// Filename, used for error reporting, source-type inference, and as the
  /// `source_path` handed to `execute`. Defaults to `"unnamed"`.
  pub filename: Option<String>,
  /// Language override: `"js"`, `"jsx"`, `"ts"`, `"tsx"` or `"dts"`.
  pub lang: Option<String>,
  /// Module-kind override: `"script"`, `"module"`, `"commonjs"` or
  /// `"unambiguous"`.
  pub source_type: Option<String>,
}

/// Render accumulated diagnostics into a single thrown error.
///
/// Goes through oxc's own `OxcError` so the message carries the labelled
/// codeframe (source excerpt + carets) rather than a bare one-line diagnostic.
fn diagnostics_to_error(filename: &str, source_text: &str, diagnostics: Diagnostics) -> NapiError {
  let errors = OxcError::from_diagnostics(filename, source_text, diagnostics);
  let mut rendered = String::new();
  for error in errors {
    let OxcError {
      message, codeframe, ..
    } = error;
    if !rendered.is_empty() {
      rendered.push_str("\n\n");
    }
    rendered.push_str(codeframe.as_deref().unwrap_or(message.as_str()));
  }
  if rendered.is_empty() {
    rendered.push_str("Compilation failed");
  }
  NapiError::from_reason(rendered)
}

/// NAPI wrapper around [`oxc::Compiler`].
///
/// JS: `const compiler = new Compiler();`
/// `compiler.execute(src, { filename: "app.ts" })`
#[napi]
pub struct Compiler {
  inner: OxcCompiler,
}

#[napi]
impl Compiler {
  /// Create a compiler running oxc's default pipeline.
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: OxcCompiler::default(),
    }
  }

  /// Run the pipeline over `source_text` and return the printed code.
  ///
  /// Throws with the rendered diagnostics (codeframes included) when the source
  /// has any syntax or semantic error; returns the printed source otherwise.
  #[napi]
  pub fn execute(
    &mut self,
    source_text: String,
    options: Option<CompilerOptions>,
  ) -> NapiResult<String> {
    let (filename, lang, source_type) = match options {
      Some(o) => (o.filename, o.lang, o.source_type),
      None => (None, None, None),
    };
    let filename = filename.unwrap_or_else(|| DEFAULT_FILENAME.to_string());

    let source_type: SourceType =
      get_source_type(&filename, lang.as_deref(), source_type.as_deref());

    self
      .inner
      .execute(&source_text, source_type, Path::new(&filename))
      .map_err(|diagnostics| diagnostics_to_error(&filename, &source_text, diagnostics))
  }
}
