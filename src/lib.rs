use napi_derive::napi;

pub mod engine;
pub mod future;
pub mod global;
pub mod parser;
mod poll;
mod result;

#[napi]
pub fn add(left: i32, right: i32) -> i32 {
  left + right
}
