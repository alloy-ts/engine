
use napi_derive::napi;

pub mod context;
pub mod future;
pub mod engine;
pub mod loader;
pub mod parser;
pub mod runtime;
mod poll;
mod result;

#[napi]
pub fn add(left: i32, right: i32) -> i32 {
  left + right
}
