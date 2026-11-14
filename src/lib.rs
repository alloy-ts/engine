//! Alloy's embedded ECMAScript runtime (a growing subset of ECMA-262).
//!
//! This crate is a pure Rust library: it has no napi or Node dependency and
//! contains no `unsafe` code. Host conversion between `napi_value`s and
//! runtime [`Value`]s happens only in the NAPI facade in alloy-ts/core.

mod completion;
mod context;
mod environment;
mod evaluator;
mod heap;
mod lexer;
mod number;
mod operations;
mod parser;
mod realm;
mod script;
mod value;

pub use completion::{Completion, CompletionRecord, CompletionType, ErrorKind, Exception};
pub use context::{Disposed, RuntimeContext};
pub use evaluator::evaluate_script;
pub use heap::Heap;
pub use number::number_to_string;
pub use realm::Realm;
pub use script::ScriptRecord;
pub use value::{StringHandle, Value};
