//! Alloy's embedded ECMAScript runtime (MVP subset).
//!
//! Nothing in this module depends on napi: host conversion happens only in
//! `crate::engine`.

mod completion;
mod context;
mod evaluator;
mod heap;
mod lexer;
mod number;
mod operations;
mod parser;
mod realm;
mod script;
mod value;

pub use completion::{Completion, ErrorKind, Exception};
pub use context::{Disposed, RuntimeContext};
pub use evaluator::evaluate_script;
pub use heap::Heap;
pub use number::number_to_string;
pub use realm::Realm;
pub use script::ScriptRecord;
pub use value::{StringHandle, Value};
