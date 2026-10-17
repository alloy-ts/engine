use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::{atom::PredefinedAtom, Atom as QjsAtom};
use serde_json::Value as JsonValue;

use crate::context::JsContext;

fn u32_to_predefined_atom(id: u32) -> Option<PredefinedAtom> {
  match id {
    x if x == PredefinedAtom::Null as u32 => Some(PredefinedAtom::Null),
    x if x == PredefinedAtom::False as u32 => Some(PredefinedAtom::False),
    x if x == PredefinedAtom::True as u32 => Some(PredefinedAtom::True),
    x if x == PredefinedAtom::If as u32 => Some(PredefinedAtom::If),
    x if x == PredefinedAtom::Else as u32 => Some(PredefinedAtom::Else),
    x if x == PredefinedAtom::Return as u32 => Some(PredefinedAtom::Return),
    x if x == PredefinedAtom::Var as u32 => Some(PredefinedAtom::Var),
    x if x == PredefinedAtom::This as u32 => Some(PredefinedAtom::This),
    x if x == PredefinedAtom::Delete as u32 => Some(PredefinedAtom::Delete),
    x if x == PredefinedAtom::Void as u32 => Some(PredefinedAtom::Void),
    x if x == PredefinedAtom::Typeof as u32 => Some(PredefinedAtom::Typeof),
    x if x == PredefinedAtom::New as u32 => Some(PredefinedAtom::New),
    x if x == PredefinedAtom::In as u32 => Some(PredefinedAtom::In),
    x if x == PredefinedAtom::Instanceof as u32 => Some(PredefinedAtom::Instanceof),
    x if x == PredefinedAtom::Do as u32 => Some(PredefinedAtom::Do),
    x if x == PredefinedAtom::While as u32 => Some(PredefinedAtom::While),
    x if x == PredefinedAtom::For as u32 => Some(PredefinedAtom::For),
    x if x == PredefinedAtom::Break as u32 => Some(PredefinedAtom::Break),
    x if x == PredefinedAtom::Continue as u32 => Some(PredefinedAtom::Continue),
    x if x == PredefinedAtom::Switch as u32 => Some(PredefinedAtom::Switch),
    x if x == PredefinedAtom::Case as u32 => Some(PredefinedAtom::Case),
    x if x == PredefinedAtom::Default as u32 => Some(PredefinedAtom::Default),
    x if x == PredefinedAtom::Throw as u32 => Some(PredefinedAtom::Throw),
    x if x == PredefinedAtom::Try as u32 => Some(PredefinedAtom::Try),
    x if x == PredefinedAtom::Catch as u32 => Some(PredefinedAtom::Catch),
    x if x == PredefinedAtom::Finally as u32 => Some(PredefinedAtom::Finally),
    x if x == PredefinedAtom::FunctionKw as u32 => Some(PredefinedAtom::FunctionKw),
    x if x == PredefinedAtom::Debugger as u32 => Some(PredefinedAtom::Debugger),
    x if x == PredefinedAtom::With as u32 => Some(PredefinedAtom::With),
    x if x == PredefinedAtom::Class as u32 => Some(PredefinedAtom::Class),
    x if x == PredefinedAtom::Const as u32 => Some(PredefinedAtom::Const),
    x if x == PredefinedAtom::Enum as u32 => Some(PredefinedAtom::Enum),
    x if x == PredefinedAtom::Export as u32 => Some(PredefinedAtom::Export),
    x if x == PredefinedAtom::Extends as u32 => Some(PredefinedAtom::Extends),
    x if x == PredefinedAtom::Import as u32 => Some(PredefinedAtom::Import),
    x if x == PredefinedAtom::Super as u32 => Some(PredefinedAtom::Super),
    x if x == PredefinedAtom::Implements as u32 => Some(PredefinedAtom::Implements),
    x if x == PredefinedAtom::Interface as u32 => Some(PredefinedAtom::Interface),
    x if x == PredefinedAtom::Let as u32 => Some(PredefinedAtom::Let),
    x if x == PredefinedAtom::Package as u32 => Some(PredefinedAtom::Package),
    x if x == PredefinedAtom::Private as u32 => Some(PredefinedAtom::Private),
    x if x == PredefinedAtom::Protected as u32 => Some(PredefinedAtom::Protected),
    x if x == PredefinedAtom::Public as u32 => Some(PredefinedAtom::Public),
    x if x == PredefinedAtom::Static as u32 => Some(PredefinedAtom::Static),
    x if x == PredefinedAtom::Yield as u32 => Some(PredefinedAtom::Yield),
    x if x == PredefinedAtom::Await as u32 => Some(PredefinedAtom::Await),
    x if x == PredefinedAtom::Empty as u32 => Some(PredefinedAtom::Empty),
    x if x == PredefinedAtom::Length as u32 => Some(PredefinedAtom::Length),
    x if x == PredefinedAtom::Message as u32 => Some(PredefinedAtom::Message),
    x if x == PredefinedAtom::Errors as u32 => Some(PredefinedAtom::Errors),
    x if x == PredefinedAtom::Stack as u32 => Some(PredefinedAtom::Stack),
    x if x == PredefinedAtom::Name as u32 => Some(PredefinedAtom::Name),
    x if x == PredefinedAtom::ToString as u32 => Some(PredefinedAtom::ToString),
    x if x == PredefinedAtom::ToLocaleString as u32 => Some(PredefinedAtom::ToLocaleString),
    x if x == PredefinedAtom::ValueOf as u32 => Some(PredefinedAtom::ValueOf),
    x if x == PredefinedAtom::Eval as u32 => Some(PredefinedAtom::Eval),
    x if x == PredefinedAtom::Prototype as u32 => Some(PredefinedAtom::Prototype),
    x if x == PredefinedAtom::Constructor as u32 => Some(PredefinedAtom::Constructor),
    x if x == PredefinedAtom::Configurable as u32 => Some(PredefinedAtom::Configurable),
    x if x == PredefinedAtom::Writable as u32 => Some(PredefinedAtom::Writable),
    x if x == PredefinedAtom::Enumerable as u32 => Some(PredefinedAtom::Enumerable),
    x if x == PredefinedAtom::Value as u32 => Some(PredefinedAtom::Value),
    x if x == PredefinedAtom::Getter as u32 => Some(PredefinedAtom::Getter),
    x if x == PredefinedAtom::Setter as u32 => Some(PredefinedAtom::Setter),
    x if x == PredefinedAtom::Of as u32 => Some(PredefinedAtom::Of),
    x if x == PredefinedAtom::UnderscoreProto as u32 => Some(PredefinedAtom::UnderscoreProto),
    x if x == PredefinedAtom::Undefined as u32 => Some(PredefinedAtom::Undefined),
    x if x == PredefinedAtom::NumberLower as u32 => Some(PredefinedAtom::NumberLower),
    x if x == PredefinedAtom::BooleanLower as u32 => Some(PredefinedAtom::BooleanLower),
    x if x == PredefinedAtom::StringLower as u32 => Some(PredefinedAtom::StringLower),
    x if x == PredefinedAtom::ObjectLower as u32 => Some(PredefinedAtom::ObjectLower),
    x if x == PredefinedAtom::SymbolLower as u32 => Some(PredefinedAtom::SymbolLower),
    x if x == PredefinedAtom::Integer as u32 => Some(PredefinedAtom::Integer),
    x if x == PredefinedAtom::Unknown as u32 => Some(PredefinedAtom::Unknown),
    x if x == PredefinedAtom::ArgumentsLower as u32 => Some(PredefinedAtom::ArgumentsLower),
    x if x == PredefinedAtom::Callee as u32 => Some(PredefinedAtom::Callee),
    x if x == PredefinedAtom::Caller as u32 => Some(PredefinedAtom::Caller),
    x if x == PredefinedAtom::LastIndex as u32 => Some(PredefinedAtom::LastIndex),
    x if x == PredefinedAtom::Target as u32 => Some(PredefinedAtom::Target),
    x if x == PredefinedAtom::Index as u32 => Some(PredefinedAtom::Index),
    x if x == PredefinedAtom::Input as u32 => Some(PredefinedAtom::Input),
    x if x == PredefinedAtom::DefineProperties as u32 => Some(PredefinedAtom::DefineProperties),
    x if x == PredefinedAtom::Apply as u32 => Some(PredefinedAtom::Apply),
    x if x == PredefinedAtom::Join as u32 => Some(PredefinedAtom::Join),
    x if x == PredefinedAtom::Concat as u32 => Some(PredefinedAtom::Concat),
    x if x == PredefinedAtom::Split as u32 => Some(PredefinedAtom::Split),
    x if x == PredefinedAtom::Construct as u32 => Some(PredefinedAtom::Construct),
    x if x == PredefinedAtom::GetPrototypeOf as u32 => Some(PredefinedAtom::GetPrototypeOf),
    x if x == PredefinedAtom::SetPrototypeOf as u32 => Some(PredefinedAtom::SetPrototypeOf),
    x if x == PredefinedAtom::IsExtensible as u32 => Some(PredefinedAtom::IsExtensible),
    x if x == PredefinedAtom::PreventExtensions as u32 => Some(PredefinedAtom::PreventExtensions),
    x if x == PredefinedAtom::Has as u32 => Some(PredefinedAtom::Has),
    x if x == PredefinedAtom::DeleteProperty as u32 => Some(PredefinedAtom::DeleteProperty),
    x if x == PredefinedAtom::DefineProperty as u32 => Some(PredefinedAtom::DefineProperty),
    x if x == PredefinedAtom::GetOwnPropertyDescriptor as u32 => Some(PredefinedAtom::GetOwnPropertyDescriptor),
    x if x == PredefinedAtom::OwnKeys as u32 => Some(PredefinedAtom::OwnKeys),
    x if x == PredefinedAtom::Add as u32 => Some(PredefinedAtom::Add),
    x if x == PredefinedAtom::Done as u32 => Some(PredefinedAtom::Done),
    x if x == PredefinedAtom::Next as u32 => Some(PredefinedAtom::Next),
    x if x == PredefinedAtom::Values as u32 => Some(PredefinedAtom::Values),
    x if x == PredefinedAtom::Source as u32 => Some(PredefinedAtom::Source),
    x if x == PredefinedAtom::Flags as u32 => Some(PredefinedAtom::Flags),
    x if x == PredefinedAtom::Global as u32 => Some(PredefinedAtom::Global),
    x if x == PredefinedAtom::Unicode as u32 => Some(PredefinedAtom::Unicode),
    x if x == PredefinedAtom::Raw as u32 => Some(PredefinedAtom::Raw),
    x if x == PredefinedAtom::NewTarget as u32 => Some(PredefinedAtom::NewTarget),
    x if x == PredefinedAtom::ThisActiveFunc as u32 => Some(PredefinedAtom::ThisActiveFunc),
    x if x == PredefinedAtom::HomeObject as u32 => Some(PredefinedAtom::HomeObject),
    x if x == PredefinedAtom::ComputedField as u32 => Some(PredefinedAtom::ComputedField),
    x if x == PredefinedAtom::StaticComputedField as u32 => Some(PredefinedAtom::StaticComputedField),
    x if x == PredefinedAtom::ClassFieldsInit as u32 => Some(PredefinedAtom::ClassFieldsInit),
    x if x == PredefinedAtom::Brand as u32 => Some(PredefinedAtom::Brand),
    x if x == PredefinedAtom::HashConstructor as u32 => Some(PredefinedAtom::HashConstructor),
    x if x == PredefinedAtom::As as u32 => Some(PredefinedAtom::As),
    x if x == PredefinedAtom::From as u32 => Some(PredefinedAtom::From),
    x if x == PredefinedAtom::Meta as u32 => Some(PredefinedAtom::Meta),
    x if x == PredefinedAtom::StarDefault as u32 => Some(PredefinedAtom::StarDefault),
    x if x == PredefinedAtom::Star as u32 => Some(PredefinedAtom::Star),
    x if x == PredefinedAtom::Module as u32 => Some(PredefinedAtom::Module),
    x if x == PredefinedAtom::Then as u32 => Some(PredefinedAtom::Then),
    x if x == PredefinedAtom::Resolve as u32 => Some(PredefinedAtom::Resolve),
    x if x == PredefinedAtom::Reject as u32 => Some(PredefinedAtom::Reject),
    x if x == PredefinedAtom::PromiseLower as u32 => Some(PredefinedAtom::PromiseLower),
    x if x == PredefinedAtom::ProxyLower as u32 => Some(PredefinedAtom::ProxyLower),
    x if x == PredefinedAtom::Revoke as u32 => Some(PredefinedAtom::Revoke),
    x if x == PredefinedAtom::Async as u32 => Some(PredefinedAtom::Async),
    x if x == PredefinedAtom::Exec as u32 => Some(PredefinedAtom::Exec),
    x if x == PredefinedAtom::Groups as u32 => Some(PredefinedAtom::Groups),
    x if x == PredefinedAtom::Status as u32 => Some(PredefinedAtom::Status),
    x if x == PredefinedAtom::Reason as u32 => Some(PredefinedAtom::Reason),
    x if x == PredefinedAtom::GlobalThis as u32 => Some(PredefinedAtom::GlobalThis),
    x if x == PredefinedAtom::Bigint as u32 => Some(PredefinedAtom::Bigint),
    x if x == PredefinedAtom::ToJSON as u32 => Some(PredefinedAtom::ToJSON),
    x if x == PredefinedAtom::MaxByteLength as u32 => Some(PredefinedAtom::MaxByteLength),
    x if x == PredefinedAtom::Object as u32 => Some(PredefinedAtom::Object),
    x if x == PredefinedAtom::Array as u32 => Some(PredefinedAtom::Array),
    x if x == PredefinedAtom::Error as u32 => Some(PredefinedAtom::Error),
    x if x == PredefinedAtom::Number as u32 => Some(PredefinedAtom::Number),
    x if x == PredefinedAtom::String as u32 => Some(PredefinedAtom::String),
    x if x == PredefinedAtom::Boolean as u32 => Some(PredefinedAtom::Boolean),
    x if x == PredefinedAtom::Symbol as u32 => Some(PredefinedAtom::Symbol),
    x if x == PredefinedAtom::Arguments as u32 => Some(PredefinedAtom::Arguments),
    x if x == PredefinedAtom::Math as u32 => Some(PredefinedAtom::Math),
    x if x == PredefinedAtom::JSON as u32 => Some(PredefinedAtom::JSON),
    x if x == PredefinedAtom::Date as u32 => Some(PredefinedAtom::Date),
    x if x == PredefinedAtom::Function as u32 => Some(PredefinedAtom::Function),
    x if x == PredefinedAtom::GeneratorFunction as u32 => Some(PredefinedAtom::GeneratorFunction),
    x if x == PredefinedAtom::ForInIterator as u32 => Some(PredefinedAtom::ForInIterator),
    x if x == PredefinedAtom::Iterator as u32 => Some(PredefinedAtom::Iterator),
    x if x == PredefinedAtom::RegExp as u32 => Some(PredefinedAtom::RegExp),
    x if x == PredefinedAtom::ArrayBuffer as u32 => Some(PredefinedAtom::ArrayBuffer),
    x if x == PredefinedAtom::SharedArrayBuffer as u32 => Some(PredefinedAtom::SharedArrayBuffer),
    x if x == PredefinedAtom::Uint8ClampedArray as u32 => Some(PredefinedAtom::Uint8ClampedArray),
    x if x == PredefinedAtom::Int8Array as u32 => Some(PredefinedAtom::Int8Array),
    x if x == PredefinedAtom::Uint8Array as u32 => Some(PredefinedAtom::Uint8Array),
    x if x == PredefinedAtom::Int16Array as u32 => Some(PredefinedAtom::Int16Array),
    x if x == PredefinedAtom::Uint16Array as u32 => Some(PredefinedAtom::Uint16Array),
    x if x == PredefinedAtom::Int32Array as u32 => Some(PredefinedAtom::Int32Array),
    x if x == PredefinedAtom::Uint32Array as u32 => Some(PredefinedAtom::Uint32Array),
    x if x == PredefinedAtom::BigInt64Array as u32 => Some(PredefinedAtom::BigInt64Array),
    x if x == PredefinedAtom::BigUint64Array as u32 => Some(PredefinedAtom::BigUint64Array),
    x if x == PredefinedAtom::Float32Array as u32 => Some(PredefinedAtom::Float32Array),
    x if x == PredefinedAtom::Float64Array as u32 => Some(PredefinedAtom::Float64Array),
    x if x == PredefinedAtom::DataView as u32 => Some(PredefinedAtom::DataView),
    x if x == PredefinedAtom::BigInt as u32 => Some(PredefinedAtom::BigInt),
    x if x == PredefinedAtom::Map as u32 => Some(PredefinedAtom::Map),
    x if x == PredefinedAtom::Set as u32 => Some(PredefinedAtom::Set),
    x if x == PredefinedAtom::WeakMap as u32 => Some(PredefinedAtom::WeakMap),
    x if x == PredefinedAtom::WeakSet as u32 => Some(PredefinedAtom::WeakSet),
    x if x == PredefinedAtom::MapIterator as u32 => Some(PredefinedAtom::MapIterator),
    x if x == PredefinedAtom::SetIterator as u32 => Some(PredefinedAtom::SetIterator),
    x if x == PredefinedAtom::ArrayIterator as u32 => Some(PredefinedAtom::ArrayIterator),
    x if x == PredefinedAtom::StringIterator as u32 => Some(PredefinedAtom::StringIterator),
    x if x == PredefinedAtom::RegExpStringIterator as u32 => Some(PredefinedAtom::RegExpStringIterator),
    x if x == PredefinedAtom::Generator as u32 => Some(PredefinedAtom::Generator),
    x if x == PredefinedAtom::Proxy as u32 => Some(PredefinedAtom::Proxy),
    x if x == PredefinedAtom::Promise as u32 => Some(PredefinedAtom::Promise),
    x if x == PredefinedAtom::PromiseResolveFunction as u32 => Some(PredefinedAtom::PromiseResolveFunction),
    x if x == PredefinedAtom::PromiseRejectFunction as u32 => Some(PredefinedAtom::PromiseRejectFunction),
    x if x == PredefinedAtom::AsyncFunction as u32 => Some(PredefinedAtom::AsyncFunction),
    x if x == PredefinedAtom::AsyncFunctionResolve as u32 => Some(PredefinedAtom::AsyncFunctionResolve),
    x if x == PredefinedAtom::AsyncFunctionReject as u32 => Some(PredefinedAtom::AsyncFunctionReject),
    x if x == PredefinedAtom::AsyncGeneratorFunction as u32 => Some(PredefinedAtom::AsyncGeneratorFunction),
    x if x == PredefinedAtom::AsyncGenerator as u32 => Some(PredefinedAtom::AsyncGenerator),
    x if x == PredefinedAtom::EvalError as u32 => Some(PredefinedAtom::EvalError),
    x if x == PredefinedAtom::RangeError as u32 => Some(PredefinedAtom::RangeError),
    x if x == PredefinedAtom::ReferenceError as u32 => Some(PredefinedAtom::ReferenceError),
    x if x == PredefinedAtom::SyntaxError as u32 => Some(PredefinedAtom::SyntaxError),
    x if x == PredefinedAtom::TypeError as u32 => Some(PredefinedAtom::TypeError),
    x if x == PredefinedAtom::URIError as u32 => Some(PredefinedAtom::URIError),
    x if x == PredefinedAtom::InternalError as u32 => Some(PredefinedAtom::InternalError),
    x if x == PredefinedAtom::DOMException as u32 => Some(PredefinedAtom::DOMException),
    x if x == PredefinedAtom::SymbolAsyncIterator as u32 => Some(PredefinedAtom::SymbolAsyncIterator),
    x if x == PredefinedAtom::SymbolIterator as u32 => Some(PredefinedAtom::SymbolIterator),
    x if x == PredefinedAtom::SymbolMatch as u32 => Some(PredefinedAtom::SymbolMatch),
    x if x == PredefinedAtom::SymbolMatchAll as u32 => Some(PredefinedAtom::SymbolMatchAll),
    x if x == PredefinedAtom::SymbolReplace as u32 => Some(PredefinedAtom::SymbolReplace),
    x if x == PredefinedAtom::SymbolSearch as u32 => Some(PredefinedAtom::SymbolSearch),
    x if x == PredefinedAtom::SymbolSplit as u32 => Some(PredefinedAtom::SymbolSplit),
    x if x == PredefinedAtom::SymbolToPrimitive as u32 => Some(PredefinedAtom::SymbolToPrimitive),
    x if x == PredefinedAtom::SymbolToStringTag as u32 => Some(PredefinedAtom::SymbolToStringTag),
    x if x == PredefinedAtom::SymbolIsConcatSpreadable as u32 => Some(PredefinedAtom::SymbolIsConcatSpreadable),
    x if x == PredefinedAtom::SymbolHasInstance as u32 => Some(PredefinedAtom::SymbolHasInstance),
    x if x == PredefinedAtom::SymbolSpecies as u32 => Some(PredefinedAtom::SymbolSpecies),
    x if x == PredefinedAtom::SymbolUnscopables as u32 => Some(PredefinedAtom::SymbolUnscopables),
    _ => None,
  }
}

#[napi(js_name = "Atom")]
pub struct Atom {
  pub value: String,
}

#[napi]
impl Atom {
  /// Create an Atom from a JS Value in a given Context.
  #[napi(factory, js_name = "from_value")]
  pub fn from_value(ctx: &JsContext, val: JsonValue) -> Result<Self> {
    match val {
      JsonValue::Null => Self::from_str(ctx, "null".to_string()),
      JsonValue::Bool(b) => Self::from_bool(ctx, b),
      JsonValue::Number(ref n) => {
        if let Some(u) = n.as_u64() {
          if u <= u32::MAX as u64 {
            return Self::from_u32(ctx, u as u32);
          }
        }
        if let Some(i) = n.as_i64() {
          if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
            return Self::from_i32(ctx, i as i32);
          }
        }
        if let Some(f) = n.as_f64() {
          return Self::from_f64(ctx, f);
        }
        Self::from_str(ctx, n.to_string())
      }
      JsonValue::String(s) => Self::from_str(ctx, s),
      JsonValue::Array(_) | JsonValue::Object(_) => {
        let json_str = serde_json::to_string(&val).map_err(|e| Error::from_reason(e.to_string()))?;
        ctx.inner.with(|c| {
          let qjs_val = c
            .json_parse(json_str)
            .map_err(|e| Error::from_reason(e.to_string()))?;
          let atom = QjsAtom::from_value(c.clone(), &qjs_val)
            .map_err(|e| Error::from_reason(e.to_string()))?;
          let val_str = atom
            .to_string()
            .map_err(|e| Error::from_reason(e.to_string()))?;
          Ok(Self { value: val_str })
        })
      }
    }
  }

  /// Alias for `from_value` in camelCase for JS convenience.
  #[napi(factory, js_name = "fromValue")]
  pub fn from_value_camel(ctx: &JsContext, val: JsonValue) -> Result<Self> {
    Self::from_value(ctx, val)
  }

  /// Create an Atom from a u32 integer in a given Context.
  #[napi(factory, js_name = "from_u32")]
  pub fn from_u32(ctx: &JsContext, val: u32) -> Result<Self> {
    ctx.inner.with(|c| {
      let atom = QjsAtom::from_u32(c.clone(), val)
        .map_err(|e| Error::from_reason(e.to_string()))?;
      let val_str = atom
        .to_string()
        .map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { value: val_str })
    })
  }

  /// Alias for `from_u32` in camelCase for JS convenience.
  #[napi(factory, js_name = "fromU32")]
  pub fn from_u32_camel(ctx: &JsContext, val: u32) -> Result<Self> {
    Self::from_u32(ctx, val)
  }

  /// Create an Atom from an i32 integer in a given Context.
  #[napi(factory, js_name = "from_i32")]
  pub fn from_i32(ctx: &JsContext, val: i32) -> Result<Self> {
    ctx.inner.with(|c| {
      let atom = QjsAtom::from_i32(c.clone(), val)
        .map_err(|e| Error::from_reason(e.to_string()))?;
      let val_str = atom
        .to_string()
        .map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { value: val_str })
    })
  }

  /// Alias for `from_i32` in camelCase for JS convenience.
  #[napi(factory, js_name = "fromI32")]
  pub fn from_i32_camel(ctx: &JsContext, val: i32) -> Result<Self> {
    Self::from_i32(ctx, val)
  }

  /// Create an Atom from a bool in a given Context.
  #[napi(factory, js_name = "from_bool")]
  pub fn from_bool(ctx: &JsContext, val: bool) -> Result<Self> {
    ctx.inner.with(|c| {
      let atom = QjsAtom::from_bool(c.clone(), val)
        .map_err(|e| Error::from_reason(e.to_string()))?;
      let val_str = atom
        .to_string()
        .map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { value: val_str })
    })
  }

  /// Alias for `from_bool` in camelCase for JS convenience.
  #[napi(factory, js_name = "fromBool")]
  pub fn from_bool_camel(ctx: &JsContext, val: bool) -> Result<Self> {
    Self::from_bool(ctx, val)
  }

  /// Create an Atom from a f64 in a given Context.
  #[napi(factory, js_name = "from_f64")]
  pub fn from_f64(ctx: &JsContext, val: f64) -> Result<Self> {
    ctx.inner.with(|c| {
      let atom = QjsAtom::from_f64(c.clone(), val)
        .map_err(|e| Error::from_reason(e.to_string()))?;
      let val_str = atom
        .to_string()
        .map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { value: val_str })
    })
  }

  /// Alias for `from_f64` in camelCase for JS convenience.
  #[napi(factory, js_name = "fromF64")]
  pub fn from_f64_camel(ctx: &JsContext, val: f64) -> Result<Self> {
    Self::from_f64(ctx, val)
  }

  /// Create an Atom from a string in a given Context.
  #[napi(factory, js_name = "from_str")]
  pub fn from_str(ctx: &JsContext, name: String) -> Result<Self> {
    ctx.inner.with(|c| {
      let atom = QjsAtom::from_str(c.clone(), &name)
        .map_err(|e| Error::from_reason(e.to_string()))?;
      let val_str = atom
        .to_string()
        .map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { value: val_str })
    })
  }

  /// Alias for `from_str` in camelCase for JS convenience.
  #[napi(factory, js_name = "fromStr")]
  pub fn from_str_camel(ctx: &JsContext, name: String) -> Result<Self> {
    Self::from_str(ctx, name)
  }

  /// Create an Atom from a predefined atom in a given Context.
  #[napi(factory, js_name = "from_predefined")]
  pub fn from_predefined(ctx: &JsContext, predefined: Either<u32, String>) -> Result<Self> {
    ctx.inner.with(|c| {
      let atom = match predefined {
        Either::A(id) => {
          let predef = u32_to_predefined_atom(id)
            .ok_or_else(|| Error::from_reason(format!("Invalid predefined atom ID: {id}")))?;
          QjsAtom::from_predefined(c.clone(), predef)
        }
        Either::B(name) => QjsAtom::from_str(c.clone(), &name)
          .map_err(|e| Error::from_reason(e.to_string()))?,
      };
      let val_str = atom
        .to_string()
        .map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(Self { value: val_str })
    })
  }

  /// Alias for `from_predefined` in camelCase for JS convenience.
  #[napi(factory, js_name = "fromPredefined")]
  pub fn from_predefined_camel(ctx: &JsContext, predefined: Either<u32, String>) -> Result<Self> {
    Self::from_predefined(ctx, predefined)
  }

  /// Convert the Atom to a JavaScript string.
  #[napi(js_name = "to_string")]
  pub fn to_string(&self) -> String {
    self.value.clone()
  }

  /// Alias for `to_string` standard toString in JS.
  #[napi(js_name = "toString")]
  pub fn to_string_camel(&self) -> String {
    self.value.clone()
  }

  /// Convert the Atom to a JavaScript string.
  #[napi(js_name = "to_js_string")]
  pub fn to_js_string(&self) -> String {
    self.value.clone()
  }

  /// Alias for `to_js_string` in camelCase for JS convenience.
  #[napi(js_name = "toJsString")]
  pub fn to_js_string_camel(&self) -> String {
    self.value.clone()
  }

  /// Convert the Atom to a JavaScript value.
  #[napi(js_name = "to_value")]
  pub fn to_value(&self) -> JsonValue {
    JsonValue::String(self.value.clone())
  }

  /// Alias for `to_value` in camelCase for JS convenience.
  #[napi(js_name = "toValue")]
  pub fn to_value_camel(&self) -> JsonValue {
    JsonValue::String(self.value.clone())
  }
}
