use napi_derive::napi;

pub mod compiler;
pub mod context;
pub mod engine;
pub mod global;
pub mod loader;
pub mod parser;

pub use compiler::Compiler;
pub use context::Context;
pub use engine::JsEngine;
pub use global::Global;
pub use loader::Loader;
pub use parser::Parser;

#[napi]
pub fn add(left: i32, right: i32) -> i32 {
  left + right
}
