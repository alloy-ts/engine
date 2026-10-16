//! NAPI bindings and QuickJS Atom wrapper.

use std::fmt;
use napi_derive::napi;

/// NAPI wrapper around a QuickJS Atom.
///
/// In QuickJS, atoms are similar to interned strings but with additional uses.
/// Symbols for instance are represented as atoms.
#[napi(js_name = "Atom")]
pub struct JsAtom {
    pub(crate) name: String,
}

#[napi]
impl JsAtom {
    /// Create a JSAtom from a string representation.
    #[napi(constructor)]
    pub fn new(name: String) -> Self {
        JsAtom { name }
    }

    /// Convert the atom to its string value.
    #[napi(js_name = "toString")]
    pub fn to_js_string(&self) -> String {
        self.name.clone()
    }
}

impl fmt::Display for JsAtom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
