use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::atom::PredefinedAtom as QjsPredefinedAtom;
use rquickjs::Atom as QjsAtom;
use rquickjs::String as QjsString;
use rquickjs::Value as QjsValue;
use serde_json::json;

use crate::context::{to_napi_err, JsContext};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtomKind {
  String,
  Number,
  Boolean,
  Null,
  Value,
}

#[napi]
pub enum NativePrimitive {
  U32,
  I32,
  U64,
  I64,
  F64,
  F32,
  BOOL,
  STRING,
}

#[napi]
pub enum PredefinedAtom {
  Null = 0,
  False = 1,
  True = 2,
  If = 3,
  Else = 4,
  Return = 5,
  Var = 6,
  This = 7,
  Delete = 8,
  Void = 9,
  Typeof = 10,
  New = 11,
  In = 12,
  Instanceof = 13,
  Do = 14,
  While = 15,
  For = 16,
  Break = 17,
  Continue = 18,
  Switch = 19,
  Case = 20,
  Default = 21,
  Throw = 22,
  Try = 23,
  Catch = 24,
  Finally = 25,
  FunctionKw = 26,
  Debugger = 27,
  With = 28,
  Class = 29,
  Const = 30,
  Enum = 31,
  Export = 32,
  Extends = 33,
  Import = 34,
  Super = 35,
  Implements = 36,
  Interface = 37,
  Let = 38,
  Package = 39,
  Private = 40,
  Protected = 41,
  Public = 42,
  Static = 43,
  Yield = 44,
  Await = 45,
  Empty = 46,
  Length = 47,
  Message = 48,
  Errors = 49,
  Stack = 50,
  Name = 51,
  ToString = 52,
  ToLocaleString = 53,
  ValueOf = 54,
  Eval = 55,
  Prototype = 56,
  Constructor = 57,
  Configurable = 58,
  Writable = 59,
  Enumerable = 60,
  Value = 61,
  Getter = 62,
  Setter = 63,
  Of = 64,
  UnderscoreProto = 65,
  Undefined = 66,
  NumberLower = 67,
  BooleanLower = 68,
  StringLower = 69,
  ObjectLower = 70,
  SymbolLower = 71,
  Integer = 72,
  Unknown = 73,
  ArgumentsLower = 74,
  Callee = 75,
  Caller = 76,
  LastIndex = 77,
  Target = 78,
  Index = 79,
  Input = 80,
  DefineProperties = 81,
  Apply = 82,
  Join = 83,
  Concat = 84,
  Split = 85,
  Construct = 86,
  GetPrototypeOf = 87,
  SetPrototypeOf = 88,
  IsExtensible = 89,
  PreventExtensions = 90,
  Has = 91,
  DeleteProperty = 92,
  DefineProperty = 93,
  GetOwnPropertyDescriptor = 94,
  OwnKeys = 95,
  Add = 96,
  Done = 97,
  Next = 98,
  Values = 99,
  Source = 100,
  Flags = 101,
  Global = 102,
  Unicode = 103,
  Raw = 104,
  NewTarget = 105,
  ThisActiveFunc = 106,
  HomeObject = 107,
  ComputedField = 108,
  StaticComputedField = 109,
  ClassFieldsInit = 110,
  Brand = 111,
  HashConstructor = 112,
  As = 113,
  From = 114,
  Meta = 115,
  StarDefault = 116,
  Star = 117,
  Module = 118,
  Then = 119,
  Resolve = 120,
  Reject = 121,
  PromiseLower = 122,
  ProxyLower = 123,
  Revoke = 124,
  Async = 125,
  Exec = 126,
  Groups = 127,
  Status = 128,
  Reason = 129,
  GlobalThis = 130,
  Bigint = 131,
  ToJSON = 132,
  MaxByteLength = 133,
  Object = 134,
  Array = 135,
  Error = 136,
  Number = 137,
  String = 138,
  Boolean = 139,
  Symbol = 140,
  Arguments = 141,
  Math = 142,
  JSON = 143,
  Date = 144,
  Function = 145,
  GeneratorFunction = 146,
  ForInIterator = 147,
  Iterator = 148,
  RegExp = 149,
  ArrayBuffer = 150,
  SharedArrayBuffer = 151,
  Uint8ClampedArray = 152,
  Int8Array = 153,
  Uint8Array = 154,
  Int16Array = 155,
  Uint16Array = 156,
  Int32Array = 157,
  Uint32Array = 158,
  BigInt64Array = 159,
  BigUint64Array = 160,
  Float32Array = 161,
  Float64Array = 162,
  DataView = 163,
  BigInt = 164,
  Map = 165,
  Set = 166,
  WeakMap = 167,
  WeakSet = 168,
  MapIterator = 169,
  SetIterator = 170,
  ArrayIterator = 171,
  StringIterator = 172,
  RegExpStringIterator = 173,
  Generator = 174,
  Proxy = 175,
  Promise = 176,
  PromiseResolveFunction = 177,
  PromiseRejectFunction = 178,
  AsyncFunction = 179,
  AsyncFunctionResolve = 180,
  AsyncFunctionReject = 181,
  AsyncGeneratorFunction = 182,
  AsyncGenerator = 183,
  EvalError = 184,
  RangeError = 185,
  ReferenceError = 186,
  SyntaxError = 187,
  TypeError = 188,
  URIError = 189,
  InternalError = 190,
  DOMException = 191,
  SymbolAsyncIterator = 192,
  SymbolIterator = 193,
  SymbolMatch = 194,
  SymbolMatchAll = 195,
  SymbolReplace = 196,
  SymbolSearch = 197,
  SymbolSplit = 198,
  SymbolToPrimitive = 199,
  SymbolToStringTag = 200,
  SymbolIsConcatSpreadable = 201,
  SymbolHasInstance = 202,
  SymbolSpecies = 203,
  SymbolUnscopables = 204,
}

fn predefined_from_u32(u: u32) -> Option<QjsPredefinedAtom> {
  match u {
    0 => Some(QjsPredefinedAtom::Null),
    1 => Some(QjsPredefinedAtom::False),
    2 => Some(QjsPredefinedAtom::True),
    3 => Some(QjsPredefinedAtom::If),
    4 => Some(QjsPredefinedAtom::Else),
    5 => Some(QjsPredefinedAtom::Return),
    6 => Some(QjsPredefinedAtom::Var),
    7 => Some(QjsPredefinedAtom::This),
    8 => Some(QjsPredefinedAtom::Delete),
    9 => Some(QjsPredefinedAtom::Void),
    10 => Some(QjsPredefinedAtom::Typeof),
    11 => Some(QjsPredefinedAtom::New),
    12 => Some(QjsPredefinedAtom::In),
    13 => Some(QjsPredefinedAtom::Instanceof),
    14 => Some(QjsPredefinedAtom::Do),
    15 => Some(QjsPredefinedAtom::While),
    16 => Some(QjsPredefinedAtom::For),
    17 => Some(QjsPredefinedAtom::Break),
    18 => Some(QjsPredefinedAtom::Continue),
    19 => Some(QjsPredefinedAtom::Switch),
    20 => Some(QjsPredefinedAtom::Case),
    21 => Some(QjsPredefinedAtom::Default),
    22 => Some(QjsPredefinedAtom::Throw),
    23 => Some(QjsPredefinedAtom::Try),
    24 => Some(QjsPredefinedAtom::Catch),
    25 => Some(QjsPredefinedAtom::Finally),
    26 => Some(QjsPredefinedAtom::FunctionKw),
    27 => Some(QjsPredefinedAtom::Debugger),
    28 => Some(QjsPredefinedAtom::With),
    29 => Some(QjsPredefinedAtom::Class),
    30 => Some(QjsPredefinedAtom::Const),
    31 => Some(QjsPredefinedAtom::Enum),
    32 => Some(QjsPredefinedAtom::Export),
    33 => Some(QjsPredefinedAtom::Extends),
    34 => Some(QjsPredefinedAtom::Import),
    35 => Some(QjsPredefinedAtom::Super),
    36 => Some(QjsPredefinedAtom::Implements),
    37 => Some(QjsPredefinedAtom::Interface),
    38 => Some(QjsPredefinedAtom::Let),
    39 => Some(QjsPredefinedAtom::Package),
    40 => Some(QjsPredefinedAtom::Private),
    41 => Some(QjsPredefinedAtom::Protected),
    42 => Some(QjsPredefinedAtom::Public),
    43 => Some(QjsPredefinedAtom::Static),
    44 => Some(QjsPredefinedAtom::Yield),
    45 => Some(QjsPredefinedAtom::Await),
    46 => Some(QjsPredefinedAtom::Empty),
    47 => Some(QjsPredefinedAtom::Length),
    48 => Some(QjsPredefinedAtom::Message),
    49 => Some(QjsPredefinedAtom::Errors),
    50 => Some(QjsPredefinedAtom::Stack),
    51 => Some(QjsPredefinedAtom::Name),
    52 => Some(QjsPredefinedAtom::ToString),
    53 => Some(QjsPredefinedAtom::ToLocaleString),
    54 => Some(QjsPredefinedAtom::ValueOf),
    55 => Some(QjsPredefinedAtom::Eval),
    56 => Some(QjsPredefinedAtom::Prototype),
    57 => Some(QjsPredefinedAtom::Constructor),
    58 => Some(QjsPredefinedAtom::Configurable),
    59 => Some(QjsPredefinedAtom::Writable),
    60 => Some(QjsPredefinedAtom::Enumerable),
    61 => Some(QjsPredefinedAtom::Value),
    62 => Some(QjsPredefinedAtom::Getter),
    63 => Some(QjsPredefinedAtom::Setter),
    64 => Some(QjsPredefinedAtom::Of),
    65 => Some(QjsPredefinedAtom::UnderscoreProto),
    66 => Some(QjsPredefinedAtom::Undefined),
    67 => Some(QjsPredefinedAtom::NumberLower),
    68 => Some(QjsPredefinedAtom::BooleanLower),
    69 => Some(QjsPredefinedAtom::StringLower),
    70 => Some(QjsPredefinedAtom::ObjectLower),
    71 => Some(QjsPredefinedAtom::SymbolLower),
    72 => Some(QjsPredefinedAtom::Integer),
    73 => Some(QjsPredefinedAtom::Unknown),
    74 => Some(QjsPredefinedAtom::ArgumentsLower),
    75 => Some(QjsPredefinedAtom::Callee),
    76 => Some(QjsPredefinedAtom::Caller),
    77 => Some(QjsPredefinedAtom::LastIndex),
    78 => Some(QjsPredefinedAtom::Target),
    79 => Some(QjsPredefinedAtom::Index),
    80 => Some(QjsPredefinedAtom::Input),
    81 => Some(QjsPredefinedAtom::DefineProperties),
    82 => Some(QjsPredefinedAtom::Apply),
    83 => Some(QjsPredefinedAtom::Join),
    84 => Some(QjsPredefinedAtom::Concat),
    85 => Some(QjsPredefinedAtom::Split),
    86 => Some(QjsPredefinedAtom::Construct),
    87 => Some(QjsPredefinedAtom::GetPrototypeOf),
    88 => Some(QjsPredefinedAtom::SetPrototypeOf),
    89 => Some(QjsPredefinedAtom::IsExtensible),
    90 => Some(QjsPredefinedAtom::PreventExtensions),
    91 => Some(QjsPredefinedAtom::Has),
    92 => Some(QjsPredefinedAtom::DeleteProperty),
    93 => Some(QjsPredefinedAtom::DefineProperty),
    94 => Some(QjsPredefinedAtom::GetOwnPropertyDescriptor),
    95 => Some(QjsPredefinedAtom::OwnKeys),
    96 => Some(QjsPredefinedAtom::Add),
    97 => Some(QjsPredefinedAtom::Done),
    98 => Some(QjsPredefinedAtom::Next),
    99 => Some(QjsPredefinedAtom::Values),
    100 => Some(QjsPredefinedAtom::Source),
    101 => Some(QjsPredefinedAtom::Flags),
    102 => Some(QjsPredefinedAtom::Global),
    103 => Some(QjsPredefinedAtom::Unicode),
    104 => Some(QjsPredefinedAtom::Raw),
    105 => Some(QjsPredefinedAtom::NewTarget),
    106 => Some(QjsPredefinedAtom::ThisActiveFunc),
    107 => Some(QjsPredefinedAtom::HomeObject),
    108 => Some(QjsPredefinedAtom::ComputedField),
    109 => Some(QjsPredefinedAtom::StaticComputedField),
    110 => Some(QjsPredefinedAtom::ClassFieldsInit),
    111 => Some(QjsPredefinedAtom::Brand),
    112 => Some(QjsPredefinedAtom::HashConstructor),
    113 => Some(QjsPredefinedAtom::As),
    114 => Some(QjsPredefinedAtom::From),
    115 => Some(QjsPredefinedAtom::Meta),
    116 => Some(QjsPredefinedAtom::StarDefault),
    117 => Some(QjsPredefinedAtom::Star),
    118 => Some(QjsPredefinedAtom::Module),
    119 => Some(QjsPredefinedAtom::Then),
    120 => Some(QjsPredefinedAtom::Resolve),
    121 => Some(QjsPredefinedAtom::Reject),
    122 => Some(QjsPredefinedAtom::PromiseLower),
    123 => Some(QjsPredefinedAtom::ProxyLower),
    124 => Some(QjsPredefinedAtom::Revoke),
    125 => Some(QjsPredefinedAtom::Async),
    126 => Some(QjsPredefinedAtom::Exec),
    127 => Some(QjsPredefinedAtom::Groups),
    128 => Some(QjsPredefinedAtom::Status),
    129 => Some(QjsPredefinedAtom::Reason),
    130 => Some(QjsPredefinedAtom::GlobalThis),
    131 => Some(QjsPredefinedAtom::Bigint),
    132 => Some(QjsPredefinedAtom::ToJSON),
    133 => Some(QjsPredefinedAtom::MaxByteLength),
    134 => Some(QjsPredefinedAtom::Object),
    135 => Some(QjsPredefinedAtom::Array),
    136 => Some(QjsPredefinedAtom::Error),
    137 => Some(QjsPredefinedAtom::Number),
    138 => Some(QjsPredefinedAtom::String),
    139 => Some(QjsPredefinedAtom::Boolean),
    140 => Some(QjsPredefinedAtom::Symbol),
    141 => Some(QjsPredefinedAtom::Arguments),
    142 => Some(QjsPredefinedAtom::Math),
    143 => Some(QjsPredefinedAtom::JSON),
    144 => Some(QjsPredefinedAtom::Date),
    145 => Some(QjsPredefinedAtom::Function),
    146 => Some(QjsPredefinedAtom::GeneratorFunction),
    147 => Some(QjsPredefinedAtom::ForInIterator),
    148 => Some(QjsPredefinedAtom::Iterator),
    149 => Some(QjsPredefinedAtom::RegExp),
    150 => Some(QjsPredefinedAtom::ArrayBuffer),
    151 => Some(QjsPredefinedAtom::SharedArrayBuffer),
    152 => Some(QjsPredefinedAtom::Uint8ClampedArray),
    153 => Some(QjsPredefinedAtom::Int8Array),
    154 => Some(QjsPredefinedAtom::Uint8Array),
    155 => Some(QjsPredefinedAtom::Int16Array),
    156 => Some(QjsPredefinedAtom::Uint16Array),
    157 => Some(QjsPredefinedAtom::Int32Array),
    158 => Some(QjsPredefinedAtom::Uint32Array),
    159 => Some(QjsPredefinedAtom::BigInt64Array),
    160 => Some(QjsPredefinedAtom::BigUint64Array),
    161 => Some(QjsPredefinedAtom::Float32Array),
    162 => Some(QjsPredefinedAtom::Float64Array),
    163 => Some(QjsPredefinedAtom::DataView),
    164 => Some(QjsPredefinedAtom::BigInt),
    165 => Some(QjsPredefinedAtom::Map),
    166 => Some(QjsPredefinedAtom::Set),
    167 => Some(QjsPredefinedAtom::WeakMap),
    168 => Some(QjsPredefinedAtom::WeakSet),
    169 => Some(QjsPredefinedAtom::MapIterator),
    170 => Some(QjsPredefinedAtom::SetIterator),
    171 => Some(QjsPredefinedAtom::ArrayIterator),
    172 => Some(QjsPredefinedAtom::StringIterator),
    173 => Some(QjsPredefinedAtom::RegExpStringIterator),
    174 => Some(QjsPredefinedAtom::Generator),
    175 => Some(QjsPredefinedAtom::Proxy),
    176 => Some(QjsPredefinedAtom::Promise),
    177 => Some(QjsPredefinedAtom::PromiseResolveFunction),
    178 => Some(QjsPredefinedAtom::PromiseRejectFunction),
    179 => Some(QjsPredefinedAtom::AsyncFunction),
    180 => Some(QjsPredefinedAtom::AsyncFunctionResolve),
    181 => Some(QjsPredefinedAtom::AsyncFunctionReject),
    182 => Some(QjsPredefinedAtom::AsyncGeneratorFunction),
    183 => Some(QjsPredefinedAtom::AsyncGenerator),
    184 => Some(QjsPredefinedAtom::EvalError),
    185 => Some(QjsPredefinedAtom::RangeError),
    186 => Some(QjsPredefinedAtom::ReferenceError),
    187 => Some(QjsPredefinedAtom::SyntaxError),
    188 => Some(QjsPredefinedAtom::TypeError),
    189 => Some(QjsPredefinedAtom::URIError),
    190 => Some(QjsPredefinedAtom::InternalError),
    191 => Some(QjsPredefinedAtom::DOMException),
    192 => Some(QjsPredefinedAtom::SymbolAsyncIterator),
    193 => Some(QjsPredefinedAtom::SymbolIterator),
    194 => Some(QjsPredefinedAtom::SymbolMatch),
    195 => Some(QjsPredefinedAtom::SymbolMatchAll),
    196 => Some(QjsPredefinedAtom::SymbolReplace),
    197 => Some(QjsPredefinedAtom::SymbolSearch),
    198 => Some(QjsPredefinedAtom::SymbolSplit),
    199 => Some(QjsPredefinedAtom::SymbolToPrimitive),
    200 => Some(QjsPredefinedAtom::SymbolToStringTag),
    201 => Some(QjsPredefinedAtom::SymbolIsConcatSpreadable),
    202 => Some(QjsPredefinedAtom::SymbolHasInstance),
    203 => Some(QjsPredefinedAtom::SymbolSpecies),
    204 => Some(QjsPredefinedAtom::SymbolUnscopables),
    _ => None,
  }
}

#[napi(js_name = "Atom")]
pub struct JsAtom {
  pub value: String,
  pub(crate) kind: AtomKind,
}

#[napi]
impl JsAtom {
  /// Constant predefined atoms for JS under Atom.inJs.*
  #[napi(js_name = "inJs")]
  pub fn in_js() -> serde_json::Value {
    json!({
      "Null": PredefinedAtom::Null as u32,
      "False": PredefinedAtom::False as u32,
      "True": PredefinedAtom::True as u32,
      "Undefined": PredefinedAtom::Undefined as u32,
      "Length": PredefinedAtom::Length as u32,
      "Message": PredefinedAtom::Message as u32,
      "Name": PredefinedAtom::Name as u32,
      "ToString": PredefinedAtom::ToString as u32,
      "ValueOf": PredefinedAtom::ValueOf as u32,
      "Prototype": PredefinedAtom::Prototype as u32,
      "Constructor": PredefinedAtom::Constructor as u32,
      "Value": PredefinedAtom::Value as u32,
      "Object": PredefinedAtom::Object as u32,
      "Array": PredefinedAtom::Array as u32,
      "Error": PredefinedAtom::Error as u32,
      "Number": PredefinedAtom::Number as u32,
      "String": PredefinedAtom::String as u32,
      "Boolean": PredefinedAtom::Boolean as u32,
      "Symbol": PredefinedAtom::Symbol as u32,
      "Promise": PredefinedAtom::Promise as u32,
      "Function": PredefinedAtom::Function as u32,
      "GlobalThis": PredefinedAtom::GlobalThis as u32,
    })
  }

  /// Native primitive types under Atom.inRs.*
  #[napi(js_name = "inRs")]
  pub fn in_rs() -> serde_json::Value {
    json!({
      "U32": NativePrimitive::U32 as u32,
      "I32": NativePrimitive::I32 as u32,
      "U64": NativePrimitive::U64 as u32,
      "I64": NativePrimitive::I64 as u32,
      "F64": NativePrimitive::F64 as u32,
      "F32": NativePrimitive::F32 as u32,
      "BOOL": NativePrimitive::BOOL as u32,
      "STRING": NativePrimitive::STRING as u32,
    })
  }

  /// Create an atom from a JS Value
  #[napi(factory, js_name = "from_value")]
  pub fn from_value(ctx: Option<&JsContext>, val: serde_json::Value) -> napi::Result<Self> {
    let created_ctx;
    let ctx = match ctx {
      Some(c) => c,
      None => {
        created_ctx = JsContext::new()?;
        &created_ctx
      }
    };
    let kind = match &val {
      serde_json::Value::Null => AtomKind::Null,
      serde_json::Value::Bool(_) => AtomKind::Boolean,
      serde_json::Value::Number(_) => AtomKind::Number,
      serde_json::Value::String(_) => AtomKind::String,
      _ => AtomKind::Value,
    };
    ctx.inner.with(|c| {
      let qjs_val = match &val {
        serde_json::Value::Null => QjsValue::new_null(c.clone()),
        serde_json::Value::Bool(b) => QjsValue::new_bool(c.clone(), *b),
        serde_json::Value::Number(n) => {
          if let Some(i) = n.as_i64() {
            QjsValue::new_int(c.clone(), i as i32)
          } else if let Some(f) = n.as_f64() {
            QjsValue::new_float(c.clone(), f)
          } else {
            QjsValue::new_null(c.clone())
          }
        }
        serde_json::Value::String(s) => QjsString::from_str(c.clone(), s)
          .map_err(to_napi_err)?
          .into_value(),
        _ => c.json_parse(serde_json::to_string(&val).map_err(|e| Error::from_reason(e.to_string()))?)
          .map_err(to_napi_err)?,
      };
      let atom = QjsAtom::from_value(c.clone(), &qjs_val).map_err(to_napi_err)?;
      let val_str = atom.to_string().map_err(to_napi_err)?;
      Ok(Self { value: val_str, kind })
    })
  }

  #[napi(factory, js_name = "fromValue")]
  pub fn from_value_camel(ctx: Option<&JsContext>, val: serde_json::Value) -> napi::Result<Self> {
    Self::from_value(ctx, val)
  }

  /// Create an Atom from a u32
  #[napi(factory, js_name = "from_u32")]
  pub fn from_u32(ctx_or_val: Either<&JsContext, u32>, maybe_val: Option<u32>) -> napi::Result<Self> {
    let created_ctx;
    let (ctx, val) = match ctx_or_val {
      Either::A(c) => (c, maybe_val.unwrap_or(0)),
      Either::B(v) => {
        created_ctx = JsContext::new()?;
        (&created_ctx, v)
      }
    };
    ctx.inner.with(|c| {
      let atom = QjsAtom::from_u32(c.clone(), val).map_err(to_napi_err)?;
      let val_str = atom.to_string().map_err(to_napi_err)?;
      Ok(Self { value: val_str, kind: AtomKind::Number })
    })
  }

  #[napi(factory, js_name = "fromU32")]
  pub fn from_u32_camel(ctx_or_val: Either<&JsContext, u32>, maybe_val: Option<u32>) -> napi::Result<Self> {
    Self::from_u32(ctx_or_val, maybe_val)
  }

  /// Create an Atom from an i32
  #[napi(factory, js_name = "from_i32")]
  pub fn from_i32(ctx_or_val: Either<&JsContext, i32>, maybe_val: Option<i32>) -> napi::Result<Self> {
    let created_ctx;
    let (ctx, val) = match ctx_or_val {
      Either::A(c) => (c, maybe_val.unwrap_or(0)),
      Either::B(v) => {
        created_ctx = JsContext::new()?;
        (&created_ctx, v)
      }
    };
    ctx.inner.with(|c| {
      let atom = QjsAtom::from_i32(c.clone(), val).map_err(to_napi_err)?;
      let val_str = atom.to_string().map_err(to_napi_err)?;
      Ok(Self { value: val_str, kind: AtomKind::Number })
    })
  }

  #[napi(factory, js_name = "fromI32")]
  pub fn from_i32_camel(ctx_or_val: Either<&JsContext, i32>, maybe_val: Option<i32>) -> napi::Result<Self> {
    Self::from_i32(ctx_or_val, maybe_val)
  }

  /// Create an Atom from a bool
  #[napi(factory, js_name = "from_bool")]
  pub fn from_bool(ctx_or_val: Either<&JsContext, bool>, maybe_val: Option<bool>) -> napi::Result<Self> {
    let created_ctx;
    let (ctx, val) = match ctx_or_val {
      Either::A(c) => (c, maybe_val.unwrap_or(false)),
      Either::B(v) => {
        created_ctx = JsContext::new()?;
        (&created_ctx, v)
      }
    };
    ctx.inner.with(|c| {
      let atom = QjsAtom::from_bool(c.clone(), val).map_err(to_napi_err)?;
      let val_str = atom.to_string().map_err(to_napi_err)?;
      Ok(Self { value: val_str, kind: AtomKind::Boolean })
    })
  }

  #[napi(factory, js_name = "fromBool")]
  pub fn from_bool_camel(ctx_or_val: Either<&JsContext, bool>, maybe_val: Option<bool>) -> napi::Result<Self> {
    Self::from_bool(ctx_or_val, maybe_val)
  }

  /// Create an Atom from a f64
  #[napi(factory, js_name = "from_f64")]
  pub fn from_f64(ctx_or_val: Either<&JsContext, f64>, maybe_val: Option<f64>) -> napi::Result<Self> {
    let created_ctx;
    let (ctx, val) = match ctx_or_val {
      Either::A(c) => (c, maybe_val.unwrap_or(0.0)),
      Either::B(v) => {
        created_ctx = JsContext::new()?;
        (&created_ctx, v)
      }
    };
    ctx.inner.with(|c| {
      let atom = QjsAtom::from_f64(c.clone(), val).map_err(to_napi_err)?;
      let val_str = atom.to_string().map_err(to_napi_err)?;
      Ok(Self { value: val_str, kind: AtomKind::Number })
    })
  }

  #[napi(factory, js_name = "fromF64")]
  pub fn from_f64_camel(ctx_or_val: Either<&JsContext, f64>, maybe_val: Option<f64>) -> napi::Result<Self> {
    Self::from_f64(ctx_or_val, maybe_val)
  }

  /// Create an Atom from a string
  #[napi(factory, js_name = "from_str")]
  pub fn from_str(ctx_or_val: Either<&JsContext, String>, maybe_val: Option<String>) -> napi::Result<Self> {
    let created_ctx;
    let (ctx, name) = match ctx_or_val {
      Either::A(c) => (c, maybe_val.unwrap_or_default()),
      Either::B(v) => {
        created_ctx = JsContext::new()?;
        (&created_ctx, v)
      }
    };
    ctx.inner.with(|c| {
      let atom = QjsAtom::from_str(c.clone(), &name).map_err(to_napi_err)?;
      let val_str = atom.to_string().map_err(to_napi_err)?;
      Ok(Self { value: val_str, kind: AtomKind::String })
    })
  }

  #[napi(factory, js_name = "fromStr")]
  pub fn from_str_camel(ctx_or_val: Either<&JsContext, String>, maybe_val: Option<String>) -> napi::Result<Self> {
    Self::from_str(ctx_or_val, maybe_val)
  }

  /// Alias for from_str for backward compatibility
  #[napi(factory, js_name = "from_string")]
  pub fn from_string(ctx_or_val: Either<&JsContext, String>, maybe_val: Option<String>) -> napi::Result<Self> {
    Self::from_str(ctx_or_val, maybe_val)
  }

  /// Create an Atom from a predefined atom or string
  #[napi(factory, js_name = "from_predefined")]
  pub fn from_predefined(ctx: &JsContext, predefined: Either<u32, String>) -> napi::Result<Self> {
    ctx.inner.with(|c| {
      let (val_str, kind) = match predefined {
        Either::A(u) => {
          if let Some(predef) = predefined_from_u32(u) {
            let atom = QjsAtom::from_predefined(c.clone(), predef);
            (atom.to_string().map_err(to_napi_err)?, AtomKind::String)
          } else {
            let atom = QjsAtom::from_str(c.clone(), &u.to_string()).map_err(to_napi_err)?;
            (atom.to_string().map_err(to_napi_err)?, AtomKind::String)
          }
        }
        Either::B(s) => {
          let atom = QjsAtom::from_str(c.clone(), &s).map_err(to_napi_err)?;
          (atom.to_string().map_err(to_napi_err)?, AtomKind::String)
        }
      };
      Ok(Self { value: val_str, kind })
    })
  }

  #[napi(factory, js_name = "fromPredefined")]
  pub fn from_predefined_camel(ctx: &JsContext, predefined: Either<u32, String>) -> napi::Result<Self> {
    Self::from_predefined(ctx, predefined)
  }

  // Primitive type constructors
  #[napi(factory, js_name = "U32")]
  pub fn primitive_u32_upper(ctx_or_val: Either<&JsContext, u32>, maybe_val: Option<u32>) -> napi::Result<Self> {
    Self::from_u32(ctx_or_val, maybe_val)
  }

  #[napi(factory, js_name = "u32")]
  pub fn primitive_u32_lower(ctx_or_val: Either<&JsContext, u32>, maybe_val: Option<u32>) -> napi::Result<Self> {
    Self::from_u32(ctx_or_val, maybe_val)
  }

  #[napi(factory, js_name = "I32")]
  pub fn primitive_i32_upper(ctx_or_val: Either<&JsContext, i32>, maybe_val: Option<i32>) -> napi::Result<Self> {
    Self::from_i32(ctx_or_val, maybe_val)
  }

  #[napi(factory, js_name = "i32")]
  pub fn primitive_i32_lower(ctx_or_val: Either<&JsContext, i32>, maybe_val: Option<i32>) -> napi::Result<Self> {
    Self::from_i32(ctx_or_val, maybe_val)
  }

  #[napi(factory, js_name = "U64")]
  pub fn primitive_u64_upper(ctx_or_val: Either<&JsContext, f64>, maybe_val: Option<f64>) -> napi::Result<Self> {
    Self::primitive_u64_lower(ctx_or_val, maybe_val)
  }

  #[napi(factory, js_name = "u64")]
  pub fn primitive_u64_lower(ctx_or_val: Either<&JsContext, f64>, maybe_val: Option<f64>) -> napi::Result<Self> {
    let created_ctx;
    let (ctx, val) = match ctx_or_val {
      Either::A(c) => (c, maybe_val.unwrap_or(0.0)),
      Either::B(v) => {
        created_ctx = JsContext::new()?;
        (&created_ctx, v)
      }
    };
    let u = val as u64;
    if u <= u32::MAX as u64 {
      Self::from_u32(Either::A(ctx), Some(u as u32))
    } else {
      Self::from_str(Either::A(ctx), Some(u.to_string()))
    }
  }

  #[napi(factory, js_name = "I64")]
  pub fn primitive_i64_upper(ctx_or_val: Either<&JsContext, f64>, maybe_val: Option<f64>) -> napi::Result<Self> {
    Self::primitive_i64_lower(ctx_or_val, maybe_val)
  }

  #[napi(factory, js_name = "i64")]
  pub fn primitive_i64_lower(ctx_or_val: Either<&JsContext, f64>, maybe_val: Option<f64>) -> napi::Result<Self> {
    let created_ctx;
    let (ctx, val) = match ctx_or_val {
      Either::A(c) => (c, maybe_val.unwrap_or(0.0)),
      Either::B(v) => {
        created_ctx = JsContext::new()?;
        (&created_ctx, v)
      }
    };
    let i = val as i64;
    if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
      Self::from_i32(Either::A(ctx), Some(i as i32))
    } else {
      Self::from_str(Either::A(ctx), Some(i.to_string()))
    }
  }

  #[napi(factory, js_name = "F64")]
  pub fn primitive_f64_upper(ctx_or_val: Either<&JsContext, f64>, maybe_val: Option<f64>) -> napi::Result<Self> {
    Self::from_f64(ctx_or_val, maybe_val)
  }

  #[napi(factory, js_name = "f64")]
  pub fn primitive_f64_lower(ctx_or_val: Either<&JsContext, f64>, maybe_val: Option<f64>) -> napi::Result<Self> {
    Self::from_f64(ctx_or_val, maybe_val)
  }

  #[napi(factory, js_name = "F32")]
  pub fn primitive_f32_upper(ctx_or_val: Either<&JsContext, f64>, maybe_val: Option<f64>) -> napi::Result<Self> {
    Self::primitive_f32_lower(ctx_or_val, maybe_val)
  }

  #[napi(factory, js_name = "f32")]
  pub fn primitive_f32_lower(ctx_or_val: Either<&JsContext, f64>, maybe_val: Option<f64>) -> napi::Result<Self> {
    let created_ctx;
    let (ctx, val) = match ctx_or_val {
      Either::A(c) => (c, maybe_val.unwrap_or(0.0)),
      Either::B(v) => {
        created_ctx = JsContext::new()?;
        (&created_ctx, v)
      }
    };
    Self::from_f64(Either::A(ctx), Some(val as f32 as f64))
  }

  #[napi(factory, js_name = "BOOL")]
  pub fn primitive_bool_upper(ctx_or_val: Either<&JsContext, bool>, maybe_val: Option<bool>) -> napi::Result<Self> {
    Self::from_bool(ctx_or_val, maybe_val)
  }

  #[napi(factory, js_name = "bool")]
  pub fn primitive_bool_lower(ctx_or_val: Either<&JsContext, bool>, maybe_val: Option<bool>) -> napi::Result<Self> {
    Self::from_bool(ctx_or_val, maybe_val)
  }

  #[napi(factory, js_name = "STRING")]
  pub fn primitive_string_upper(ctx_or_val: Either<&JsContext, String>, maybe_val: Option<String>) -> napi::Result<Self> {
    Self::from_str(ctx_or_val, maybe_val)
  }

  #[napi(factory, js_name = "string")]
  pub fn primitive_string_lower(ctx_or_val: Either<&JsContext, String>, maybe_val: Option<String>) -> napi::Result<Self> {
    Self::from_str(ctx_or_val, maybe_val)
  }

  // Instance methods under prototype
  #[napi(js_name = "to_string")]
  pub fn to_string(&self) -> String {
    self.value.clone()
  }

  #[napi(js_name = "toString")]
  pub fn to_string_js(&self) -> String {
    self.to_string()
  }

  #[napi(js_name = "to_js_string")]
  pub fn to_js_string(&self) -> String {
    self.value.clone()
  }

  #[napi(js_name = "toJsString")]
  pub fn to_js_string_camel(&self) -> String {
    self.to_js_string()
  }

  #[napi(js_name = "to_value")]
  pub fn to_value(&self) -> serde_json::Value {
    match self.kind {
      AtomKind::String => serde_json::Value::String(self.value.clone()),
      AtomKind::Number => {
        if let Ok(i) = self.value.parse::<i64>() {
          serde_json::Value::Number(i.into())
        } else if let Ok(f) = self.value.parse::<f64>() {
          if let Some(n) = serde_json::Number::from_f64(f) {
            serde_json::Value::Number(n)
          } else {
            serde_json::Value::String(self.value.clone())
          }
        } else {
          serde_json::Value::String(self.value.clone())
        }
      }
      AtomKind::Boolean => {
        if let Ok(b) = self.value.parse::<bool>() {
          serde_json::Value::Bool(b)
        } else {
          serde_json::Value::String(self.value.clone())
        }
      }
      AtomKind::Null => serde_json::Value::Null,
      AtomKind::Value => serde_json::from_str(&self.value)
        .unwrap_or_else(|_| serde_json::Value::String(self.value.clone())),
    }
  }

  #[napi(js_name = "toValue")]
  pub fn to_value_camel(&self) -> serde_json::Value {
    self.to_value()
  }
}
