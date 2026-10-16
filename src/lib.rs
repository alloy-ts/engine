use napi_derive::napi;

pub mod context;
pub mod engine;
pub mod global;
pub mod loader;
pub mod parser;
pub mod runtime;

#[napi]
pub fn add(left: i32, right: i32) -> i32 {
  left + right
}
