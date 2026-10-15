use napi_derive::napi;

pub mod context;
pub mod engine;
pub mod realm;
pub mod global;
#[cfg(feature = "quickjs")]
pub mod loader;
pub mod parser;
pub mod script;
