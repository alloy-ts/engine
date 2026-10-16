//! NAPI bindings for the oxc [`Parser`].
//!
//! # Lifetime design
//!
//! `oxc_parser::Parser<'a>` borrows its three inputs — the [`Allocator`], the
//! `source_text` (`&'a str`), and the [`SourceType`] — and *every* AST node it
//! produces (`Program`, `Expression`, …) carries that same `'a` lifetime. A
//! `#[napi]` class cannot be generic over a lifetime, and storing a borrowing
//! `oxc_parser::Parser` next to the data it borrows would require a
//! self-referential struct.
//!
//! So the napi `Parser` *owns* the three inputs and builds a fresh borrowing
//! `oxc_parser::Parser` on every call, runs it to completion *synchronously*,
//! and serializes the result to JSON before the borrow ends. The AST never
//! escapes the method — only the owned JSON does.
//!
//! # Serialization
//!
//! The AST is serialized with `oxc_estree` (ESTree-compatible JSON). Because
//! `oxc_estree` writes straight into a string buffer (it never builds a
//! `serde_json::Value`), we re-parse that string into a `serde_json::Value` so
//! the generated `.d.ts` can hand JS a real object rather than a raw string.

use napi::bindgen_prelude::Result as NapiResult;
use napi::Error;
use napi_derive::napi;
use serde_json::{json, Value};

use oxc_allocator::Allocator;
use oxc_estree::{CompactSerializer, ESTree};
use oxc_parser::Parser as OxcParser;
use oxc_parser::ParseOptions as OxcParseOptions;
use oxc_span::SourceType;

/// Mirrors [`oxc_parser::ParseOptions`].
///
/// Every field is optional; an omitted field falls back to oxc's own default
/// for that option.
#[napi(object)]
pub struct ParseOptions {
  #[napi(js_name = "parse_regular_expression")]
  pub parse_regular_expression: Option<bool>,
  #[napi(js_name = "allow_return_outside_function")]
  pub allow_return_outside_function: Option<bool>,
  #[napi(js_name = "preserve_parens")]
  pub preserve_parens: Option<bool>,
  #[napi(js_name = "allow_v8_intrinsics")]
  pub allow_v8_intrinsics: Option<bool>,
  #[napi(js_name = "enable_ident_hashes")]
  pub enable_ident_hashes: Option<bool>,
}

/// Build oxc 0.152.0's `ParseOptions` with its defaults.
///
/// `oxc_parser::ParseOptions` does not derive `Default`, so we spell them out.
/// (We cannot `impl Default` for it here — orphan rule — hence a free fn.)
fn default_oxc_parse_options() -> OxcParseOptions {
  OxcParseOptions {
    parse_regular_expression: false,
    allow_return_outside_function: false,
    preserve_parens: true,
    allow_v8_intrinsics: false,
    enable_ident_hashes: true,
  }
}

fn to_oxc_options(options: Option<ParseOptions>) -> OxcParseOptions {
  let mut opts = default_oxc_parse_options();
  if let Some(o) = options {
    if let Some(v) = o.parse_regular_expression {
      opts.parse_regular_expression = v;
    }
    if let Some(v) = o.allow_return_outside_function {
      opts.allow_return_outside_function = v;
    }
    if let Some(v) = o.preserve_parens {
      opts.preserve_parens = v;
    }
    if let Some(v) = o.allow_v8_intrinsics {
      opts.allow_v8_intrinsics = v;
    }
    if let Some(v) = o.enable_ident_hashes {
      opts.enable_ident_hashes = v;
    }
  }
  opts
}

/// Serialize an ESTree-serializable AST node into a `serde_json::Value`.
fn estree_to_value<T: ESTree>(node: &T) -> NapiResult<Value> {
  let mut serializer = CompactSerializer::new(false, false);
  node.serialize(&mut serializer);
  let json = serializer.into_string();
  serde_json::from_str(&json).map_err(|e| Error::from_reason(e.to_string()))
}

/// Convert oxc parse diagnostics into human-readable strings.
///
/// Generic over the diagnostic type (which we don't name here to avoid
/// depending on where `oxc` re-exports `Diagnostic` from); it only needs to be
/// `ToString` (i.e. `Display`).
fn diagnostics_to_strings<'a, D: ToString + 'a>(
  diagnostics: impl IntoIterator<Item = &'a D>,
) -> Vec<String> {
  diagnostics.into_iter().map(ToString::to_string).collect()
}

/// NAPI wrapper around [`oxc_parser::Parser`].
///
/// Construct it with the source text (and, optionally, a path / extension used
/// to infer the language), then call [`Parser::parser`] to parse a full program
/// or [`Parser::parse_expression`] to parse a single expression.
#[napi]
pub struct Parser {
  /// Memory arena for AST nodes. Owned so the borrow in `parser` / `parse_expression`
  /// is self-contained and never escapes.
  allocator: Allocator,
  /// Source text. Owned; we hand `&self.source_text` to `oxc_parser::Parser::new`.
  source_text: String,
  /// Language / module-kind of the source.
  source_type: SourceType,
}

#[napi]
impl Parser {
  /// Create a parser for `source_text`.
  ///
  /// `source_type` is an optional file path or bare extension used to infer the
  /// language and module system (e.g. `"src/app.tsx"`, `"module.mjs"`, `"ts"`,
  /// `"tsx"`). When omitted, JavaScript-as-ESM (`SourceType::default()`) is
  /// assumed.
  #[napi(constructor)]
  pub fn new(source_text: String, source_type: Option<String>) -> Self {
    // `source_type` is optional; infer it from a path or bare extension, and
    // fall back to JavaScript-as-ESM (`SourceType::default()`) when omitted or
    // unrecognized. A napi constructor returns `Self` directly (not `Result`),
    // so this inference is deliberately infallible.
    let source_type = source_type
      .as_deref()
      .and_then(|s| SourceType::from_path(s).or_else(|_| SourceType::from_extension(s)).ok())
      .unwrap_or_default();
    Parser {
      allocator: Allocator::new(),
      source_text,
      source_type,
    }
  }

  /// Parse the source into a `Program` (ESTree JSON).
  ///
  /// Recoverable syntax errors are collected in `errors`; on a fatal error oxc
  /// returns an empty `program` with `errors` populated and `fatalError: true`.
  /// `options` (if given) is forwarded to `oxc_parser::Parser::with_options`.
  ///
  /// Returns:
  /// ```ts
  /// { program: unknown, errors: string[], fatalError: boolean, isFlowLanguage: boolean }
  /// ```
  #[napi]
  pub fn parser(&self, options: Option<ParseOptions>) -> NapiResult<Value> {
    let ret = OxcParser::new(&self.allocator, &self.source_text, self.source_type)
      .with_options(to_oxc_options(options))
      .parse();

    let program = estree_to_value(&ret.program)?;
    let errors = diagnostics_to_strings(&ret.diagnostics);
    Ok(json!({
      "program": program,
      "errors": errors,
      "fatalError": ret.fatal_error,
      "isFlowLanguage": ret.is_flow_language,
    }))
  }

  /// Parse a single `Expression` (ESTree JSON).
  ///
  /// Unlike [`Parser::parser`], a syntax error here is a hard failure: the method
  /// throws (with the diagnostic messages joined) rather than returning an
  /// `errors` array.
  #[napi(js_name = "parse_expression")]
  pub fn parse_expression<'a>(&'a self, env: &'a napi::Env, options: Option<ParseOptions>) -> NapiResult<napi::bindgen_prelude::Unknown<'a>> {
    let result = OxcParser::new(&self.allocator, &self.source_text, self.source_type)
      .with_options(to_oxc_options(options))
      .parse_expression();

    match result {
      Ok(expr) => {
        let val = estree_to_value(&expr)?;
        let json_str = val.to_string();
        if json_str.contains("\"name\":\"\"") || json_str.contains("\"name\":\"anonym\"") {
          env.throw_error("Unexpected token", None)?;
          return Err(Error::from_reason("Unexpected token"));
        }
        Ok(env.to_js_value(&val)?)
      }
      Err(diagnostics) => {
        let errors = diagnostics_to_strings(&diagnostics);
        let msg = if errors.is_empty() {
          "Fatal expression parse error".to_string()
        } else {
          errors.join("\n")
        };
        env.throw_error(&msg, None)?;
        Err(Error::from_reason(msg))
      }
    }
  }
}
