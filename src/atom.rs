//! NAPI bindings for QuickJS atoms.
//!
//! In QuickJS atoms are similar to interned strings but with additional uses.
//! A symbol for instance is just an atom.
//!
//! # Representation
//!
//! Atoms in QuickJS are handled differently depending on what type of index
//! they represent. When the atom represents a number-like index, like
//! `object[1]`, the atom is just a normal number. However when the atom
//! represents a string-like index like `object["foo"]` or `object.foo` the
//! atom is a value in a hashmap.
//!
//! # Lifetime design
//!
//! A `#[napi]` class cannot be generic over a lifetime, so the native [`Atom`]
//! below cannot hold the borrowing `rquickjs::Atom<'js>` (nor the `Ctx<'js>`
//! that atom borrows from). Instead it owns:
//!
//! - a clone of the embedded QuickJS [`Context`](rquickjs::Context) — a cheap
//!   refcount bump that keeps the realm, and the runtime behind it, alive and
//!   lets every method re-enter it through `Context::with`; and
//! - the raw `JSAtom` handle, of which it owns a single reference that is
//!   released when the JavaScript object is collected.
//!
//! Numeric and predefined atoms are constants inside QuickJS, so dup/free are
//! no-ops for them and no refcounting happens for those.

use core::{ffi::CStr, ptr::null_mut};

use napi::bindgen_prelude::{BigInt, ClassInstance, Env, Error, JsObjectValue, Object, Result};
use napi_derive::napi;
use rquickjs::{qjs, Context as QjsContext, Ctx, Value};
use serde_json::Value as JsonValue;

use crate::context::{to_napi_err, value_to_json, JsContext};

/// `JS_ATOM_TAG_INT` — the bit QuickJS sets on number-like atoms instead of
/// interning them (see the module docs). The macro lives in `quickjs.c` and is
/// not re-exported by `rquickjs-sys`, so it is spelled out here.
const JS_ATOM_TAG_INT: qjs::JSAtom = 1 << 31;

/// Render a QuickJS exception that was already taken out of its context.
fn exception_to_string<'js>(ctx: &Ctx<'js>, ex: Value<'js>) -> String {
  value_to_json(ctx, ex)
    .ok()
    .and_then(|json| serde_json::to_string(&json).ok())
    .unwrap_or_else(|| "unknown exception".to_string())
}

/// Generate the [`PredefinedAtom`] enum together with its `JSAtom` mapping.
///
/// napi enums require literal variant values, which the `JS_ATOM_*` constants
/// are not, so the QuickJS predefined atoms get their own
/// `#[napi(string_enum)]` — whose JavaScript values are the variant names —
/// plus a hand-written mapping to the raw atom each variant stands for. The
/// mapping references the `rquickjs-sys` constants directly, so there are no
/// magic numbers to drift out of sync with the embedded QuickJS.
macro_rules! predefined_atoms {
  ($($variant:ident => $js_atom:ident),* $(,)?) => {
    /// An atom QuickJS predefined for itself.
    ///
    /// Predefined atoms cover the language keywords, the well-known property
    /// names (`length`, `prototype`, …) and the well-known symbols
    /// (`Symbol.iterator`, …). Using these over
    /// [`Atom::from_str`](crate::Atom::from_str) can be more performant as
    /// they don't need to be looked up in a hashmap.
    #[napi(string_enum)]
    pub enum PredefinedAtom {
      $($variant,)*
    }

    impl PredefinedAtom {
      /// The raw `JSAtom` this variant stands for.
      fn to_js_atom(self) -> qjs::JSAtom {
        match self {
          $(PredefinedAtom::$variant => qjs::$js_atom,)*
        }
      }
    }
  };
}

predefined_atoms! {
    Null => JS_ATOM_null,
    False => JS_ATOM_false,
    True => JS_ATOM_true,
    If => JS_ATOM_if,
    Else => JS_ATOM_else,
    Return => JS_ATOM_return,
    Var => JS_ATOM_var,
    This => JS_ATOM_this,
    Delete => JS_ATOM_delete,
    Void => JS_ATOM_void,
    Typeof => JS_ATOM_typeof,
    New => JS_ATOM_new,
    In => JS_ATOM_in,
    Instanceof => JS_ATOM_instanceof,
    Do => JS_ATOM_do,
    While => JS_ATOM_while,
    For => JS_ATOM_for,
    Break => JS_ATOM_break,
    Continue => JS_ATOM_continue,
    Switch => JS_ATOM_switch,
    Case => JS_ATOM_case,
    Default => JS_ATOM_default,
    Throw => JS_ATOM_throw,
    Try => JS_ATOM_try,
    Catch => JS_ATOM_catch,
    Finally => JS_ATOM_finally,
    FunctionKw => JS_ATOM_function,
    Debugger => JS_ATOM_debugger,
    With => JS_ATOM_with,
    Class => JS_ATOM_class,
    Const => JS_ATOM_const,
    Enum => JS_ATOM_enum,
    Export => JS_ATOM_export,
    Extends => JS_ATOM_extends,
    Import => JS_ATOM_import,
    Super => JS_ATOM_super,
    Implements => JS_ATOM_implements,
    Interface => JS_ATOM_interface,
    Let => JS_ATOM_let,
    Package => JS_ATOM_package,
    Private => JS_ATOM_private,
    Protected => JS_ATOM_protected,
    Public => JS_ATOM_public,
    Static => JS_ATOM_static,
    Yield => JS_ATOM_yield,
    Await => JS_ATOM_await,
    Empty => JS_ATOM_empty_string,
    Length => JS_ATOM_length,
    Message => JS_ATOM_message,
    Errors => JS_ATOM_errors,
    Stack => JS_ATOM_stack,
    Name => JS_ATOM_name,
    ToString => JS_ATOM_toString,
    ToLocaleString => JS_ATOM_toLocaleString,
    ValueOf => JS_ATOM_valueOf,
    Eval => JS_ATOM_eval,
    Prototype => JS_ATOM_prototype,
    Constructor => JS_ATOM_constructor,
    Configurable => JS_ATOM_configurable,
    Writable => JS_ATOM_writable,
    Enumerable => JS_ATOM_enumerable,
    Value => JS_ATOM_value,
    Getter => JS_ATOM_get,
    Setter => JS_ATOM_set,
    Of => JS_ATOM_of,
    UnderscoreProto => JS_ATOM___proto__,
    Undefined => JS_ATOM_undefined,
    NumberLower => JS_ATOM_number,
    BooleanLower => JS_ATOM_boolean,
    StringLower => JS_ATOM_string,
    ObjectLower => JS_ATOM_object,
    SymbolLower => JS_ATOM_symbol,
    Integer => JS_ATOM_integer,
    Unknown => JS_ATOM_unknown,
    ArgumentsLower => JS_ATOM_arguments,
    Callee => JS_ATOM_callee,
    Caller => JS_ATOM_caller,
    LastIndex => JS_ATOM_lastIndex,
    Target => JS_ATOM_target,
    Index => JS_ATOM_index,
    Input => JS_ATOM_input,
    DefineProperties => JS_ATOM_defineProperties,
    Apply => JS_ATOM_apply,
    Join => JS_ATOM_join,
    Concat => JS_ATOM_concat,
    Split => JS_ATOM_split,
    Construct => JS_ATOM_construct,
    GetPrototypeOf => JS_ATOM_getPrototypeOf,
    SetPrototypeOf => JS_ATOM_setPrototypeOf,
    IsExtensible => JS_ATOM_isExtensible,
    PreventExtensions => JS_ATOM_preventExtensions,
    Has => JS_ATOM_has,
    DeleteProperty => JS_ATOM_deleteProperty,
    DefineProperty => JS_ATOM_defineProperty,
    GetOwnPropertyDescriptor => JS_ATOM_getOwnPropertyDescriptor,
    OwnKeys => JS_ATOM_ownKeys,
    Add => JS_ATOM_add,
    Done => JS_ATOM_done,
    Next => JS_ATOM_next,
    Values => JS_ATOM_values,
    Source => JS_ATOM_source,
    Flags => JS_ATOM_flags,
    Global => JS_ATOM_global,
    Unicode => JS_ATOM_unicode,
    Raw => JS_ATOM_raw,
    NewTarget => JS_ATOM_new_target,
    ThisActiveFunc => JS_ATOM_this_active_func,
    HomeObject => JS_ATOM_home_object,
    ComputedField => JS_ATOM_computed_field,
    StaticComputedField => JS_ATOM_static_computed_field,
    ClassFieldsInit => JS_ATOM_class_fields_init,
    Brand => JS_ATOM_brand,
    HashConstructor => JS_ATOM_hash_constructor,
    As => JS_ATOM_as,
    From => JS_ATOM_from,
    Meta => JS_ATOM_meta,
    StarDefault => JS_ATOM__default_,
    Star => JS_ATOM__star_,
    Module => JS_ATOM_Module,
    Then => JS_ATOM_then,
    Resolve => JS_ATOM_resolve,
    Reject => JS_ATOM_reject,
    PromiseLower => JS_ATOM_promise,
    ProxyLower => JS_ATOM_proxy,
    Revoke => JS_ATOM_revoke,
    Async => JS_ATOM_async,
    Exec => JS_ATOM_exec,
    Groups => JS_ATOM_groups,
    Status => JS_ATOM_status,
    Reason => JS_ATOM_reason,
    GlobalThis => JS_ATOM_globalThis,
    Bigint => JS_ATOM_bigint,
    ToJSON => JS_ATOM_toJSON,
    MaxByteLength => JS_ATOM_maxByteLength,
    Object => JS_ATOM_Object,
    Array => JS_ATOM_Array,
    Error => JS_ATOM_Error,
    Number => JS_ATOM_Number,
    String => JS_ATOM_String,
    Boolean => JS_ATOM_Boolean,
    Symbol => JS_ATOM_Symbol,
    Arguments => JS_ATOM_Arguments,
    Math => JS_ATOM_Math,
    JSON => JS_ATOM_JSON,
    Date => JS_ATOM_Date,
    Function => JS_ATOM_Function,
    GeneratorFunction => JS_ATOM_GeneratorFunction,
    ForInIterator => JS_ATOM_ForInIterator,
    Iterator => JS_ATOM_Iterator,
    RegExp => JS_ATOM_RegExp,
    ArrayBuffer => JS_ATOM_ArrayBuffer,
    SharedArrayBuffer => JS_ATOM_SharedArrayBuffer,
    Uint8ClampedArray => JS_ATOM_Uint8ClampedArray,
    Int8Array => JS_ATOM_Int8Array,
    Uint8Array => JS_ATOM_Uint8Array,
    Int16Array => JS_ATOM_Int16Array,
    Uint16Array => JS_ATOM_Uint16Array,
    Int32Array => JS_ATOM_Int32Array,
    Uint32Array => JS_ATOM_Uint32Array,
    BigInt64Array => JS_ATOM_BigInt64Array,
    BigUint64Array => JS_ATOM_BigUint64Array,
    Float16Array => JS_ATOM_Float16Array,
    Float32Array => JS_ATOM_Float32Array,
    Float64Array => JS_ATOM_Float64Array,
    DataView => JS_ATOM_DataView,
    BigInt => JS_ATOM_BigInt,
    Map => JS_ATOM_Map,
    Set => JS_ATOM_Set,
    WeakMap => JS_ATOM_WeakMap,
    WeakSet => JS_ATOM_WeakSet,
    MapIterator => JS_ATOM_Map_Iterator,
    SetIterator => JS_ATOM_Set_Iterator,
    ArrayIterator => JS_ATOM_Array_Iterator,
    StringIterator => JS_ATOM_String_Iterator,
    RegExpStringIterator => JS_ATOM_RegExp_String_Iterator,
    Generator => JS_ATOM_Generator,
    Proxy => JS_ATOM_Proxy,
    Promise => JS_ATOM_Promise,
    PromiseResolveFunction => JS_ATOM_PromiseResolveFunction,
    PromiseRejectFunction => JS_ATOM_PromiseRejectFunction,
    AsyncFunction => JS_ATOM_AsyncFunction,
    AsyncFunctionResolve => JS_ATOM_AsyncFunctionResolve,
    AsyncFunctionReject => JS_ATOM_AsyncFunctionReject,
    AsyncGeneratorFunction => JS_ATOM_AsyncGeneratorFunction,
    AsyncGenerator => JS_ATOM_AsyncGenerator,
    EvalError => JS_ATOM_EvalError,
    RangeError => JS_ATOM_RangeError,
    ReferenceError => JS_ATOM_ReferenceError,
    SyntaxError => JS_ATOM_SyntaxError,
    TypeError => JS_ATOM_TypeError,
    URIError => JS_ATOM_URIError,
    InternalError => JS_ATOM_InternalError,
    DOMException => JS_ATOM_DOMException,
    SymbolAsyncIterator => JS_ATOM_Symbol_asyncIterator,
    SymbolIterator => JS_ATOM_Symbol_iterator,
    SymbolMatch => JS_ATOM_Symbol_match,
    SymbolMatchAll => JS_ATOM_Symbol_matchAll,
    SymbolReplace => JS_ATOM_Symbol_replace,
    SymbolSearch => JS_ATOM_Symbol_search,
    SymbolSplit => JS_ATOM_Symbol_split,
    SymbolToPrimitive => JS_ATOM_Symbol_toPrimitive,
    SymbolToStringTag => JS_ATOM_Symbol_toStringTag,
    SymbolIsConcatSpreadable => JS_ATOM_Symbol_isConcatSpreadable,
    SymbolHasInstance => JS_ATOM_Symbol_hasInstance,
    SymbolSpecies => JS_ATOM_Symbol_species,
    SymbolUnscopables => JS_ATOM_Symbol_unscopables,
}

/// A QuickJS Atom.
///
/// In QuickJS atoms are similar to interned strings but with additional uses.
/// A symbol for instance is just an atom.
///
/// # Representation
///
/// Atoms in QuickJS are handled differently depending on what type of index
/// they represent. When the atom represents a number-like index, like
/// `object[1]`, the atom is just a normal number. However when the atom
/// represents a string-like index like `object["foo"]` or `object.foo` the
/// atom is a value in a hashmap.
///
/// Every atom belongs to the [`Context`](crate::Context) it was created in and
/// keeps that realm alive (see the module docs).
///
/// JS:
/// ```ts
/// const ctx = new Context();
/// const length = Atom.fromStr(ctx, "length");
/// length.toString(); // "length"
/// Atom.fromU32(ctx, 1).toValue(); // 1
/// ```
#[napi]
pub struct Atom {
  /// A clone of the QuickJS context this atom belongs to. Keeps the realm (and
  /// its runtime) alive and is the only way back into it.
  ctx: QjsContext,
  /// The raw `JSAtom` handle. This struct owns a single reference to it.
  atom: qjs::JSAtom,
}

impl Drop for Atom {
  fn drop(&mut self) {
    self.ctx.with(|ctx| {
      // Safety: releases the one reference this struct owns. A no-op for
      // numeric and predefined atoms, which are QuickJS constants.
      unsafe { qjs::JS_FreeAtom(ctx.as_raw().as_ptr(), self.atom) };
    });
  }
}

#[napi]
impl Atom {
  /// `JS_AtomToString`, with a pending exception (if any) turned into an
  /// error. The returned `Value` owns the QuickJS string.
  fn atom_to_string_value<'js>(&self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
    // Safety: returns a fresh, owned `JSValue`.
    let val = unsafe { qjs::JS_AtomToString(ctx.as_raw().as_ptr(), self.atom) };
    if unsafe { qjs::JS_VALUE_GET_NORM_TAG(val) } == qjs::JS_TAG_EXCEPTION {
      let ex = ctx.catch();
      return Err(Error::from_reason(exception_to_string(ctx, ex)));
    }
    // Safety: `val` is a fresh reference, owned from here on.
    Ok(unsafe { Value::from_raw(ctx.clone(), val) })
  }

  /// Create an atom from a JavaScript value.
  ///
  /// `value` crosses the host boundary as JSON, so functions, symbols and
  /// `undefined` are rejected rather than coerced. Objects are coerced by
  /// QuickJS itself (usually to `"[object Object]"`).
  #[napi(factory)]
  pub fn from_value(ctx: &JsContext, value: JsonValue) -> Result<Self> {
    let inner = ctx.clone_inner();
    let atom = inner.with(|ctx| {
      let json = serde_json::to_string(&value).map_err(|e| Error::from_reason(e.to_string()))?;
      let value: Value = ctx.json_parse(json).map_err(to_napi_err)?;
      // Safety: `JS_ValueToAtom` borrows the value; the `Value` stays alive
      // until the end of this closure and releases it itself.
      let atom = unsafe { qjs::JS_ValueToAtom(ctx.as_raw().as_ptr(), value.as_raw()) };
      if atom == qjs::JS_ATOM_NULL {
        // Conversion failed and left an exception pending — take it so it does
        // not leak into later calls, and report it.
        let ex = ctx.catch();
        return Err(Error::from_reason(exception_to_string(&ctx, ex)));
      }
      Ok(atom)
    })?;
    Ok(Self { ctx: inner, atom })
  }

  /// Create an atom from a `u32`.
  ///
  /// Values up to `2^30 - 1` become number-like atoms — stored as the number
  /// itself, not as an interned string (see [`Atom`]); larger values are
  /// interned as their decimal string.
  #[napi(factory)]
  pub fn from_u32(ctx: &JsContext, val: u32) -> Result<Self> {
    let inner = ctx.clone_inner();
    let atom = inner.with(|ctx| {
      // Safety: `JS_NewAtomUInt32` cannot call back into JavaScript, so the
      // only failure mode is allocation failure.
      let atom = unsafe { qjs::JS_NewAtomUInt32(ctx.as_raw().as_ptr(), val) };
      if atom == qjs::JS_ATOM_NULL {
        return Err(Error::from_reason("failed to create an atom from a u32"));
      }
      Ok(atom)
    })?;
    Ok(Self { ctx: inner, atom })
  }

  /// Create an atom from an `i32` via value.
  ///
  /// Only non-negative values up to `2^30 - 1` become number-like atoms; any
  /// other value is interned as its decimal string.
  #[napi(factory)]
  pub fn from_i32(ctx: &JsContext, val: i32) -> Result<Self> {
    let inner = ctx.clone_inner();
    let atom = inner.with(|ctx| {
      let value = Value::new_int(ctx.clone(), val);
      // Safety: `JS_ValueToAtom` borrows the value; the `Value` releases it
      // when this closure ends.
      let atom = unsafe { qjs::JS_ValueToAtom(ctx.as_raw().as_ptr(), value.as_raw()) };
      if atom == qjs::JS_ATOM_NULL {
        return Err(Error::from_reason("failed to create an atom from an i32"));
      }
      Ok(atom)
    })?;
    Ok(Self { ctx: inner, atom })
  }

  /// Create an atom from a `u64`.
  ///
  /// QuickJS has no 64-bit atom, so the value is interned as its decimal
  /// string, matching how a `BigInt` property key is converted.
  #[napi(factory)]
  pub fn from_u64(ctx: &JsContext, val: BigInt) -> Result<Self> {
    let (negative, value, lossless) = val.get_u64();
    if negative || !lossless {
      return Err(Error::from_reason("value does not fit in a u64"));
    }
    Self::from_str(ctx, value.to_string())
  }

  /// Create an atom from an `i64`.
  ///
  /// QuickJS has no 64-bit atom, so the value is interned as its decimal
  /// string, matching how a `BigInt` property key is converted.
  #[napi(factory)]
  pub fn from_i64(ctx: &JsContext, val: BigInt) -> Result<Self> {
    let (value, lossless) = val.get_i64();
    if !lossless {
      return Err(Error::from_reason("value does not fit in an i64"));
    }
    Self::from_str(ctx, value.to_string())
  }

  /// Create an atom from a `f64` via value.
  ///
  /// A whole number up to `2^30 - 1` becomes a number-like atom; anything
  /// else is interned as its decimal string.
  #[napi(factory)]
  pub fn from_f64(ctx: &JsContext, val: f64) -> Result<Self> {
    let inner = ctx.clone_inner();
    let atom = inner.with(|ctx| {
      let value = Value::new_float(ctx.clone(), val);
      // Safety: `JS_ValueToAtom` borrows the value; the `Value` releases it
      // when this closure ends.
      let atom = unsafe { qjs::JS_ValueToAtom(ctx.as_raw().as_ptr(), value.as_raw()) };
      if atom == qjs::JS_ATOM_NULL {
        return Err(Error::from_reason("failed to create an atom from an f64"));
      }
      Ok(atom)
    })?;
    Ok(Self { ctx: inner, atom })
  }

  /// Create an atom from a `f32` via value.
  ///
  /// QuickJS numbers are `f64`; the value is rounded to `f32` precision first,
  /// matching what an `f32` field would hold.
  #[napi(factory)]
  pub fn from_f32(ctx: &JsContext, val: f64) -> Result<Self> {
    Self::from_f64(ctx, val as f32 as f64)
  }

  /// Create an atom from a `bool` via value.
  #[napi(factory)]
  pub fn from_bool(ctx: &JsContext, val: bool) -> Result<Self> {
    let inner = ctx.clone_inner();
    let atom = inner.with(|ctx| {
      let value = Value::new_bool(ctx.clone(), val);
      // Safety: `JS_ValueToAtom` borrows the value, which is a constant.
      let atom = unsafe { qjs::JS_ValueToAtom(ctx.as_raw().as_ptr(), value.as_raw()) };
      if atom == qjs::JS_ATOM_NULL {
        return Err(Error::from_reason("failed to create an atom from a bool"));
      }
      Ok(atom)
    })?;
    Ok(Self { ctx: inner, atom })
  }

  /// Create an atom from a Rust string.
  #[napi(factory)]
  #[allow(clippy::should_implement_trait)]
  pub fn from_str(ctx: &JsContext, name: String) -> Result<Self> {
    let inner = ctx.clone_inner();
    let atom = inner.with(|ctx| {
      // Safety: `name` outlives the call and `JS_NewAtomLen` copies what it
      // needs; it cannot call back into JavaScript.
      let atom = unsafe {
        qjs::JS_NewAtomLen(
          ctx.as_raw().as_ptr(),
          name.as_ptr().cast(),
          name.len() as qjs::size_t,
        )
      };
      if atom == qjs::JS_ATOM_NULL {
        return Err(Error::from_reason("failed to create an atom from a string"));
      }
      Ok(atom)
    })?;
    Ok(Self { ctx: inner, atom })
  }

  /// Create an atom from a predefined atom.
  #[napi(factory)]
  pub fn from_predefined(ctx: &JsContext, predefined: PredefinedAtom) -> Self {
    // Predefined atoms are QuickJS constants: no context work, no failure
    // mode. The context is still cloned so the atom can be converted later.
    Self {
      ctx: ctx.clone_inner(),
      atom: predefined.to_js_atom(),
    }
  }

  /// Convert the atom to a JavaScript string.
  #[napi]
  pub fn to_string(&self) -> Result<String> {
    self.ctx.with(|ctx| {
      // Safety: `JS_AtomToCStringLen` returns a fresh C string that must be
      // freed with `JS_FreeCString`.
      let c_str = unsafe { qjs::JS_AtomToCStringLen(ctx.as_raw().as_ptr(), null_mut(), self.atom) };
      if c_str.is_null() {
        return Err(Error::from_reason("failed to convert the atom to a string"));
      }
      let bytes = unsafe { CStr::from_ptr(c_str) }.to_bytes();
      // QuickJS strings are UTF-8; go through a lossy copy rather than
      // trusting that invariant for data this binding did not create.
      let res = String::from_utf8_lossy(bytes).into_owned();
      unsafe { qjs::JS_FreeCString(ctx.as_raw().as_ptr(), c_str) };
      Ok(res)
    })
  }

  /// Convert the atom to a JavaScript string.
  #[napi]
  pub fn to_js_string(&self) -> Result<String> {
    self.ctx.with(|ctx| {
      let value = self.atom_to_string_value(&ctx)?;
      match value_to_json(&ctx, value)? {
        JsonValue::String(string) => Ok(string),
        _ => Err(Error::from_reason("atom did not convert to a string")),
      }
    })
  }

  /// Convert the atom to a JavaScript value.
  ///
  /// Number-like atoms convert to a number; every other atom converts to a
  /// string (see [`Atom`]).
  #[napi]
  pub fn to_value(&self) -> Result<JsonValue> {
    self.ctx.with(|ctx| {
      if self.atom & JS_ATOM_TAG_INT != 0 {
        return Ok(JsonValue::from(self.atom & !JS_ATOM_TAG_INT));
      }
      let value = self.atom_to_string_value(&ctx)?;
      value_to_json(&ctx, value)
    })
  }
  #[napi(getter)]
  pub fn in_js(env: &Env) -> Result<Object<'_>> {
    let mut obj = Object::new(env)?;
    // A curated, JavaScript-facing subset of the QuickJS predefined atoms.
    macro_rules! put {
      ($($variant:ident),* $(,)?) => {
        $(obj.set_named_property(stringify!($variant), PredefinedAtom::$variant)?;)*
      };
    }
    put! {
      Null, False, True, Undefined, Empty, GlobalThis,
      If, Else, Return, Var, This, Delete, Void, Typeof, New, In, Instanceof, Do,
      While, For, Break, Continue, Switch, Case, Default, Throw, Try, Catch,
      Finally, FunctionKw, With, Class, Const, Enum, Export, Extends, Import,
      Super, Let, Static, Yield, Await,
      Length, Message, Stack, Name, ToString, ToLocaleString, ValueOf, Eval,
      Prototype, Constructor, Configurable, Writable, Enumerable, Value,
      Getter, Setter, Of, UnderscoreProto,
      Object, Array, Error, Number, String, Boolean, Symbol, Math, JSON, Date,
      Function, RegExp, Map, Set, WeakMap, WeakSet, Promise, Proxy,
      SymbolIterator, SymbolToStringTag, SymbolHasInstance, SymbolSpecies,
      SymbolUnscopables, SymbolIsConcatSpreadable,
    }
    Ok(obj)
  }

  #[napi(getter)]
  pub fn in_rs(env: &Env) -> Result<Object<'_>> {
    let mut obj = Object::new(env)?;

    let u32_fn = env.create_function_from_closure::<(), Atom, _>("U32", |ctx| {
      let jsctx = ctx.get::<ClassInstance<JsContext>>(0)?;
      let val = ctx.get::<u32>(1)?;
      Atom::from_u32(&jsctx, val)
    })?;
    obj.set_named_property("U32", u32_fn)?;

    let i32_fn = env.create_function_from_closure::<(), Atom, _>("I32", |ctx| {
      let jsctx = ctx.get::<ClassInstance<JsContext>>(0)?;
      let val = ctx.get::<i32>(1)?;
      Atom::from_i32(&jsctx, val)
    })?;
    obj.set_named_property("I32", i32_fn)?;

    let u64_fn = env.create_function_from_closure::<(), Atom, _>("U64", |ctx| {
      let jsctx = ctx.get::<ClassInstance<JsContext>>(0)?;
      let val = ctx.get::<BigInt>(1)?;
      Atom::from_u64(&jsctx, val)
    })?;
    obj.set_named_property("U64", u64_fn)?;

    let i64_fn = env.create_function_from_closure::<(), Atom, _>("I64", |ctx| {
      let jsctx = ctx.get::<ClassInstance<JsContext>>(0)?;
      let val = ctx.get::<BigInt>(1)?;
      Atom::from_i64(&jsctx, val)
    })?;
    obj.set_named_property("I64", i64_fn)?;

    let f64_fn = env.create_function_from_closure::<(), Atom, _>("F64", |ctx| {
      let jsctx = ctx.get::<ClassInstance<JsContext>>(0)?;
      let val = ctx.get::<f64>(1)?;
      Atom::from_f64(&jsctx, val)
    })?;
    obj.set_named_property("F64", f64_fn)?;

    let f32_fn = env.create_function_from_closure::<(), Atom, _>("F32", |ctx| {
      let jsctx = ctx.get::<ClassInstance<JsContext>>(0)?;
      let val = ctx.get::<f64>(1)?;
      Atom::from_f32(&jsctx, val)
    })?;
    obj.set_named_property("F32", f32_fn)?;

    let bool_fn = env.create_function_from_closure::<(), Atom, _>("BOOL", |ctx| {
      let jsctx = ctx.get::<ClassInstance<JsContext>>(0)?;
      let val = ctx.get::<bool>(1)?;
      Atom::from_bool(&jsctx, val)
    })?;
    obj.set_named_property("BOOL", bool_fn)?;

    let str_fn = env.create_function_from_closure::<(), Atom, _>("STRING", |ctx| {
      let jsctx = ctx.get::<ClassInstance<JsContext>>(0)?;
      let val = ctx.get::<String>(1)?;
      Atom::from_str(&jsctx, val)
    })?;
    obj.set_named_property("STRING", str_fn)?;

    Ok(obj)
  }
}
