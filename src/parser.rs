//! NAPI bindings for the oxc [`Parser`].

use napi::bindgen_prelude::{Error, Result};
use napi_derive::napi;
use serde_json::{json, Value};

use oxc_allocator::Allocator;
use oxc_estree::{CompactSerializer, ESTree};
use oxc_parser::Parser as OxcParser;
use oxc_parser::ParseOptions as OxcParseOptions;
use oxc_span::SourceType;

#[derive(Clone, Copy)]
#[napi(object)]
pub struct ParseOptions {
  pub parse_regular_expression: Option<bool>,
  pub allow_return_outside_function: Option<bool>,
  pub preserve_parens: Option<bool>,
  pub allow_v8_intrinsics: Option<bool>,
}

fn default_oxc_parse_options() -> OxcParseOptions {
  OxcParseOptions {
    parse_regular_expression: false,
    allow_return_outside_function: false,
    preserve_parens: true,
    allow_v8_intrinsics: false,
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
  }
  opts
}

fn estree_to_value<T: ESTree>(node: &T) -> Result<Value> {
  let mut serializer = CompactSerializer::new(false, false);
  node.serialize(&mut serializer);
  let json = serializer.into_string();
  serde_json::from_str(&json).map_err(|e| Error::from_reason(e.to_string()))
}

fn diagnostics_to_strings<'a, D: ToString + 'a>(
  diagnostics: impl IntoIterator<Item = &'a D>,
) -> Vec<String> {
  diagnostics.into_iter().map(ToString::to_string).collect()
}

#[napi]
pub struct Parser {
  allocator: Allocator,
  source_text: String,
  source_type: SourceType,
}

#[napi]
impl Parser {
  #[napi(constructor)]
  pub fn new(source_text: String, source_type: Option<String>) -> Self {
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

  #[napi]
  pub fn parser(&self, options: Option<ParseOptions>) -> Result<Value> {
    let ret = OxcParser::new(&self.allocator, &self.source_text, self.source_type)
      .with_options(to_oxc_options(options))
      .parse();

    let program = estree_to_value(&ret.program)?;
    let errors = diagnostics_to_strings(&ret.diagnostics);
    Ok(json!({
      "program": program,
      "errors": errors,
      "fatalError": ret.panicked,
      "isFlowLanguage": ret.is_flow_language,
    }))
  }

  #[napi(js_name = "parse_expression")]
  pub fn parse_expression(&self, options: Option<ParseOptions>) -> Result<Value> {
    let allocator = Allocator::new();
    let ret = OxcParser::new(&allocator, &self.source_text, self.source_type)
      .with_options(to_oxc_options(options))
      .parse();

    if !ret.diagnostics.is_empty() || ret.panicked {
      let errors = diagnostics_to_strings(&ret.diagnostics);
      let msg = if errors.is_empty() {
        "Syntax error".to_string()
      } else {
        errors.join("\n")
      };
      return Err(Error::from_reason(msg));
    }

    let allocator2 = Allocator::new();
    let expr = OxcParser::new(&allocator2, &self.source_text, self.source_type)
      .with_options(to_oxc_options(options))
      .parse_expression()
      .map_err(|e| Error::from_reason(diagnostics_to_strings(&e).join("\n")))?;

    estree_to_value(&expr)
  }
}
