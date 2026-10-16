use napi_derive::napi;

pub mod future;
pub mod engine;
pub mod global;
pub mod parser;
pub mod oxc;
pub mod compiler;
pub mod context;
mod poll;
mod result;

#[napi]
pub fn add(left: i32, right: i32) -> i32 {
  left + right
}
