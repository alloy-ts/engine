use napi_derive::napi;

pub mod atom;
pub mod compiler;
pub mod context;
pub mod engine;
pub mod global;
pub mod loader;
pub mod parser;

pub use atom::JsAtom;
pub use compiler::Compiler;

#[napi]
pub fn add(left: i32, right: i32) -> i32 {
  left + right
}
