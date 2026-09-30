
use napi_derive::napi;

pub mod future;
pub mod parser;
mod poll;
mod result;

#[napi]
pub fn add(left: i32, right: i32) -> i32 {
  left + right
}
