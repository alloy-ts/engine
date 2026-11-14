//! `@alloy-ts/engine`: Alloy TS's embedded ECMAScript runtime.
//!
//! [`runtime`] is the napi-free language runtime. `engine` is the NAPI facade
//! that exposes it to JavaScript as the `Engine` and `Context` classes.

pub mod engine;
pub mod node_api_host;
pub mod runtime;
