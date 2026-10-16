use std::path::Path;
use napi::bindgen_prelude::{Error, Result};
use napi_derive::napi;
use oxc_allocator::Allocator;
use oxc_estree::{CompactSerializer, ESTree};
use oxc_parser::Parser;
use oxc_span::SourceType;

/// NAPI wrapper around the oxc compiler.
#[napi]
pub struct Compiler;

#[napi]
impl Compiler {
  #[napi(constructor)]
  pub fn new() -> Self {
    Compiler
  }

  /// Execute compilation/transformation on `source_text`.
  #[napi]
  pub fn execute(
    &mut self,
    source_text: String,
    source_type: Option<String>,
    source_path: Option<String>,
  ) -> Result<String> {
    let path_str = source_path.as_deref().unwrap_or("index.js");
    let path = Path::new(path_str);
    let st = source_type
      .as_deref()
      .and_then(|s| SourceType::from_path(s).or_else(|_| SourceType::from_extension(s)).ok())
      .unwrap_or_else(|| SourceType::from_path(path).unwrap_or_default());

    let allocator = Allocator::new();
    let ret = Parser::new(&allocator, &source_text, st).parse();

    if !ret.diagnostics.is_empty() || ret.panicked {
      let errors: Vec<String> = ret.diagnostics.iter().map(|d| d.to_string()).collect();
      return Err(Error::from_reason(errors.join("\n")));
    }

    let mut serializer = CompactSerializer::new(false, false);
    ret.program.serialize(&mut serializer);
    Ok(serializer.into_string())
  }
}
