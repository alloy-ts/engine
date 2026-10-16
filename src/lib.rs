use napi_derive::napi;

pub mod compiler;
pub mod context;
pub mod global;
pub mod loader;
pub mod oxc;
pub mod parser;
pub mod runtime;

#[napi]
pub fn add(left: i32, right: i32) -> i32 {
  left + right
}
