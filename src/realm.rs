use std::collections::HashMap;

use super::completion::{Completion, ErrorKind, Exception};
use super::evaluator::evaluate_script;
use super::heap::Heap;
use super::lexer::is_binding_identifier;
use super::script::ScriptRecord;
use super::value::Value;

#[derive(Clone, Copy, Debug)]
struct Binding {
    value: Value,
    writable: bool,
}

/// Internal Realm: the global bindings and the heap their strings live in.
///
/// Each Realm is fully independent; Values and string handles from one Realm
/// must never be used with another.
#[derive(Debug)]
pub struct Realm {
    globals: HashMap<Box<str>, Binding>,
    heap: Heap,
}

impl Default for Realm {
    fn default() -> Self {
        Self::new()
    }
}

impl Realm {
    /// A Realm seeded with the non-writable value properties of the global
    /// object that the MVP supports (§19.1).
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        for (name, value) in [
            ("undefined", Value::Undefined),
            ("NaN", Value::Number(f64::NAN)),
            ("Infinity", Value::Number(f64::INFINITY)),
        ] {
            globals.insert(
                name.into(),
                Binding {
                    value,
                    writable: false,
                },
            );
        }
        Self {
            globals,
            heap: Heap::default(),
        }
    }

    pub fn heap(&self) -> &Heap {
        &self.heap
    }

    pub fn alloc_string(&mut self, contents: impl Into<Box<str>>) -> Value {
        Value::String(self.heap.alloc_string(contents))
    }

    pub fn get_global(&self, name: &str) -> Option<Value> {
        self.globals.get(name).map(|binding| binding.value)
    }

    /// Creates or updates a writable global binding. Fails with a TypeError
    /// for a name the evaluator could never reference or for a read-only
    /// binding, as a strict-mode assignment would.
    pub fn set_global(&mut self, name: &str, value: Value) -> Result<(), Exception> {
        if !is_binding_identifier(name) {
            return Err(Exception::new(
                ErrorKind::TypeError,
                format!("'{name}' is not a valid global binding name"),
            ));
        }
        match self.globals.get_mut(name) {
            Some(binding) if !binding.writable => Err(Exception::new(
                ErrorKind::TypeError,
                format!("Cannot assign to read only global '{name}'"),
            )),
            Some(binding) => {
                binding.value = value;
                Ok(())
            }
            None => {
                self.globals.insert(
                    name.into(),
                    Binding {
                        value,
                        writable: true,
                    },
                );
                Ok(())
            }
        }
    }

    /// Parses and evaluates `source` as a Script in this Realm.
    pub fn evaluate(&mut self, source: &str) -> Completion<Value> {
        match ScriptRecord::parse(source) {
            Ok(script) => evaluate_script(self, &script),
            Err(error) => Completion::Throw(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_globals_are_read_only() {
        let mut realm = Realm::new();
        assert!(matches!(
            realm.get_global("undefined"),
            Some(Value::Undefined)
        ));
        assert!(matches!(realm.get_global("NaN"), Some(Value::Number(n)) if n.is_nan()));
        assert!(matches!(
            realm.get_global("Infinity"),
            Some(Value::Number(f64::INFINITY))
        ));
        assert!(realm.get_global("globalThis").is_none());
        for name in ["undefined", "NaN", "Infinity"] {
            assert_eq!(
                realm.set_global(name, Value::Null).unwrap_err(),
                Exception::new(
                    ErrorKind::TypeError,
                    format!("Cannot assign to read only global '{name}'")
                )
            );
        }
        assert!(matches!(
            realm.get_global("undefined"),
            Some(Value::Undefined)
        ));
    }

    #[test]
    fn globals_are_created_and_updated() {
        let mut realm = Realm::new();
        realm.set_global("x", Value::Number(1.0)).unwrap();
        realm.set_global("x", Value::Boolean(true)).unwrap();
        assert!(matches!(realm.get_global("x"), Some(Value::Boolean(true))));
        let s = realm.alloc_string("text");
        realm.set_global("s", s).unwrap();
        let Some(Value::String(handle)) = realm.get_global("s") else {
            panic!("expected a string")
        };
        assert_eq!(realm.heap().string(handle), "text");
        for bad in ["", "1a", "a b", "true", "if"] {
            assert_eq!(
                realm.set_global(bad, Value::Null).unwrap_err().kind,
                ErrorKind::TypeError
            );
        }
    }

    #[test]
    fn realms_are_isolated() {
        let mut a = Realm::new();
        let b = Realm::new();
        a.set_global("x", Value::Number(1.0)).unwrap();
        assert!(b.get_global("x").is_none());
    }
}
