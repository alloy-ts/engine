//! NAPI bindings for the OXC Compiler.

use std::path::Path;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use oxc_allocator::Allocator;
use oxc_estree::{CompactSerializer, ESTree};
use oxc_parser::Parser as OxcParser;
use oxc_span::SourceType;

/// NAPI wrapper around the OXC compiler.
#[napi]
pub struct Compiler {
  allocator: Allocator,
}

#[napi]
impl Compiler {
  #[napi(constructor)]
  pub fn new() -> Self {
    Compiler {
      allocator: Allocator::new(),
    }
  }

  /// Execute compilation on `source_text`.
  ///
  /// `source_path` is an optional path used to derive the [`SourceType`].
  #[napi(js_name = "execute")]
  pub fn execute(
    &mut self,
    source_text: String,
    source_path: Option<String>,
  ) -> Result<String> {
    self.allocator.reset();
    let path_str = source_path.as_deref().unwrap_or("input.js");
    let path = Path::new(path_str);
    let source_type = SourceType::from_path(path).unwrap_or_default();

    let ret = OxcParser::new(&self.allocator, &source_text, source_type).parse();
    if !ret.diagnostics.is_empty() {
      let errors: Vec<String> = ret.diagnostics.iter().map(|d| d.to_string()).collect();
      return Err(Error::from_reason(errors.join("\n")));
    }

    let mut serializer = CompactSerializer::new(false, false);
    ret.program.serialize(&mut serializer);
    Ok(serializer.into_string())
  }
}
