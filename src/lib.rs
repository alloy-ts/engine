use napi_derive::napi;

pub mod context;
pub mod engine;
pub mod global;
pub mod loader;
pub mod parser;

pub use context::{Context, Ctx, EvalOptions};
pub use engine::{JsEngine as Engine, MemoryUsage, WeakEngine};
pub use loader::{
  BuiltinLoader, BuiltinResolver, JsBuiltinLoader, JsBuiltinResolver, JsFileResolver,
  JsModuleLoader, JsNativeLoader, JsScriptLoader, Loader, ModuleLoader, NativeLoader, Resolver,
};

#[napi]
pub fn add(left: i32, right: i32) -> i32 {
  left + right
}
