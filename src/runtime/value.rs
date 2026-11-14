//! The runtime's own primitive value representation.
//!
//! This is deliberately unrelated to host `napi_value`s. Only the crate's NAPI
//! facade (`src/engine.rs`) converts across that boundary.

/// Handle to a UTF-8 string owned by a Realm's [`Heap`](super::Heap).
///
/// A handle is only meaningful with the heap that allocated it. Values never
/// leave their Realm: the host boundary copies string contents out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StringHandle(pub(super) usize);

/// Handle to a Realm-owned native error record. This is the first
/// object-like runtime value and will become an ordinary Error object when
/// the object model is introduced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ErrorHandle(pub(super) usize);

/// An ECMAScript language value restricted to the MVP primitives plus a
/// Realm-owned native Error handle pending the full object model.
#[derive(Clone, Copy, Debug)]
pub enum Value {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(StringHandle),
    Error(ErrorHandle),
}
