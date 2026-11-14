//! The runtime's own primitive value representation.
//!
//! This is deliberately unrelated to the NAPI wrapper types in `crate::value`
//! and to host `napi_value`s: only `crate::engine` converts across that
//! boundary.

/// Handle to a UTF-8 string owned by a Realm's [`Heap`](super::Heap).
///
/// A handle is only meaningful with the heap that allocated it. Values never
/// leave their Realm: the host boundary copies string contents out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StringHandle(pub(super) usize);

/// An ECMAScript language value restricted to the MVP primitive types.
#[derive(Clone, Copy, Debug)]
pub enum Value {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(StringHandle),
}
