use napi_derive::napi;

pub mod compiler;
pub mod context;
pub mod engine;
pub mod global;
pub mod parser;

#[napi]
pub fn add(left: i32, right: i32) -> i32 {
  left + right
}
