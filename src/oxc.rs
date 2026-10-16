use napi::bindgen_prelude::Result as NapiResult;
use napi_derive::napi;
use serde_json::Value;

use oxc_allocator::Allocator;
use oxc_napi::{convert_utf8_to_utf16, get_source_type, Comment, OxcError};
use oxc_parser::{ParseOptions as OxcParseOptions, Parser as RawParser, ParserReturn};
use oxc_span::SourceType;

use crate::parser::{estree_to_value, to_oxc_options, ParseOptions};

const DEFAULT_FILENAME: &str = "unnamed";

#[napi(object)]
pub struct OxcParserOptions {
  /// Filename used for error reporting and source-type inference.
  /// Defaults to `"unnamed"`.
  pub filename: Option<String>,
  /// Language override: `"js"`, `"jsx"`, `"ts"`, `"tsx"` or `"dts"`.
  ///
  /// When given, oxc picks an *unambiguous* module kind so that e.g. `"ts"`
  /// behaves like a `.ts` file rather than defaulting to ESM.
  pub lang: Option<String>,
  /// Module-kind override: `"script"`, `"module"`, `"commonjs"` or
  /// `"unambiguous"`.
  pub source_type: Option<String>,
  /// Forwarded to `oxc_parser::Parser::with_options`.
  pub parse_options: Option<ParseOptions>,
}

/// Result of [`OxcParser::parse`].
#[napi(object)]
pub struct OxcParseResult {
  /// ESTree-shaped program. Every span is a **UTF-16** offset.
  pub program: Value,
  /// Comments collected from the source, with UTF-16 spans.
  pub comments: Vec<Comment>,
  /// Diagnostics, each with labelled spans and a rendered `codeframe`.
  ///
  /// May be non-empty even on success: oxc recovers from many syntax errors, so
  /// check [`fatal_error`](OxcParseResult::fatal_error) to know whether a
  /// usable AST was produced.
  pub errors: Vec<OxcError>,
  /// Whether the parser hit a fatal error and terminated early.
  ///
  /// When `true`, [`program`](OxcParseResult::program) is empty and
  /// [`errors`](OxcParseResult::errors) holds at least one entry.
  pub fatal_error: bool,
  /// Whether the source was detected as Flow.
  pub is_flow_language: bool,
}

/// NAPI wrapper parsing with oxc's own N-API helpers.
///
/// Compared to [`crate::parser::Parser`], the result carries labelled,
/// codeframed errors and comments whose spans are already converted to UTF-16,
/// i.e. directly usable as JS string indices.
#[napi]
pub struct OxcParser {
  /// Memory arena for AST nodes; owned so the borrow in `parse` stays local.
  allocator: Allocator,
  /// Source text; owned, handed to `oxc_parser::Parser::new` as `&str`.
  source_text: String,
  /// Reported in codeframes (`NamedSource`) and used for type inference.
  filename: String,
  /// Language / module-kind, resolved once via [`get_source_type`].
  source_type: SourceType,
  /// Parsing options, resolved once at construction.
  parse_options: OxcParseOptions,
}

#[napi]
impl OxcParser {
  /// Create a parser for `source_text`.
  ///
  /// `options` is optional; with none, oxc infers the source type from
  /// `filename` (defaulting to `"unnamed"`, i.e. JavaScript-as-ESM) and uses
  /// its own parser defaults.
  ///
  /// A napi constructor returns `Self` directly (not `Result`), so every
  /// decision here must be infallible — unknown `lang` / `source_type` values
  /// are simply ignored by [`get_source_type`].
  #[napi(constructor)]
  pub fn new(source_text: String, options: Option<OxcParserOptions>) -> Self {
    let (filename, lang, source_type, parse_options) = match options {
      Some(o) => (o.filename, o.lang, o.source_type, o.parse_options),
      None => (None, None, None, None),
    };
    let filename = filename.unwrap_or_else(|| DEFAULT_FILENAME.to_string());

    Self {
      allocator: Allocator::new(),
      source_text,
      // `get_source_type` is oxc's own inference: `lang` wins over the
      // extension, and `source_type` refines the module kind afterwards.
      source_type: get_source_type(&filename, lang.as_deref(), source_type.as_deref()),
      filename,
      parse_options: to_oxc_options(parse_options),
    }
  }

  /// Parse the source into a program, comments and diagnostics.
  ///
  /// Recoverable syntax errors land in `errors` alongside a usable `program`;
  /// on a fatal error `program` is empty and `fatal_error` is `true`.
  #[napi]
  pub fn parse(&self) -> NapiResult<OxcParseResult> {
    // `ParserReturn` is `#[non_exhaustive]`, so destructure with a wildcard;
    // `irregular_whitespaces` and `tokens` are unused here.
    let ParserReturn {
      mut program,
      mut module_record,
      diagnostics,
      panicked: fatal_error,
      ..
    } = RawParser::new(&self.allocator, &self.source_text, self.source_type)
      .with_options(self.parse_options)
      .parse();

    // Diagnostics -> `OxcError`s (renders a codeframe per diagnostic) *before*
    // the UTF-16 pass, which then rewrites the label offsets in place.
    // `from_diagnostics` takes owned `OxcDiagnostic`s.
    let mut errors = OxcError::from_diagnostics(&self.filename, &self.source_text, diagnostics);

    // Convert program, module record and error-label spans to UTF-16, and
    // collect comments. Must run before serialization.
    let comments = convert_utf8_to_utf16(
      &self.source_text,
      &mut program,
      &mut module_record,
      &mut errors,
    );

    Ok(OxcParseResult {
      program: estree_to_value(&program)?,
      comments,
      errors,
      fatal_error,
      is_flow_language: false,
    })
  }
}
