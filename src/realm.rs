use super::completion::{Completion, ErrorKind, Exception};
use super::environment::GlobalEnvironment;
use super::evaluator::evaluate_script;
use super::heap::Heap;
use super::lexer::is_binding_identifier;
use super::script::ScriptRecord;
use super::value::Value;

/// Internal Realm: the Global Environment Record (`[[GlobalEnv]]`) and the
/// heap its strings live in.
///
/// Each Realm is fully independent; Values and string handles from one Realm
/// must never be used with another. Bindings persist across evaluations, so
/// later Scripts see earlier declarations.
#[derive(Debug)]
pub struct Realm {
    pub(super) global_env: GlobalEnvironment,
    pub(super) heap: Heap,
}

impl Default for Realm {
    fn default() -> Self {
        Self::new()
    }
}

impl Realm {
    /// A Realm whose global object has the non-writable, non-configurable
    /// value properties the runtime supports (§19.1): `undefined`, `NaN`,
    /// `Infinity`.
    pub fn new() -> Self {
        Self {
            global_env: GlobalEnvironment::new(),
            heap: Heap::default(),
        }
    }

    pub fn heap(&self) -> &Heap {
        &self.heap
    }

    pub fn alloc_string(&mut self, contents: impl Into<Box<str>>) -> Value {
        Value::String(self.heap.alloc_string(contents))
    }

    /// The value a global identifier reference would produce: the global
    /// lexical (`let`/`const`) binding if there is one, else the global
    /// object property. A ReferenceError for a missing name or for a lexical
    /// binding still in its temporal dead zone.
    pub fn get_binding_value(&self, name: &str) -> Result<Value, Exception> {
        self.global_env.get_binding_value(name, true)
    }

    /// [`Realm::get_binding_value`] without the reason: `None` when the name
    /// is not defined or is still uninitialized.
    pub fn get_global(&self, name: &str) -> Option<Value> {
        self.get_binding_value(name).ok()
    }

    /// Assigns a global binding from the host, as a strict assignment would,
    /// except that a missing name is created as a global object property
    /// (like `globalThis[name] = value`) instead of being a ReferenceError.
    ///
    /// A TypeError for a name the evaluator could never reference, a
    /// read-only global or a `const`; a ReferenceError for a `let`/`const`
    /// still in its temporal dead zone.
    pub fn set_global(&mut self, name: &str, value: Value) -> Result<(), Exception> {
        if !is_binding_identifier(name) {
            return Err(Exception::new(
                ErrorKind::TypeError,
                format!("'{name}' is not a valid global binding name"),
            ));
        }
        if self.global_env.has_binding(name) {
            self.global_env.set_mutable_binding(name, value, true)
        } else {
            self.global_env
                .set_global_object_property(name, value, true)
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
    fn host_access_follows_global_declarations() {
        let mut realm = Realm::new();
        assert!(matches!(
            realm.evaluate("let tdz = missing; let l; const c = 1"),
            Completion::Throw(_)
        ));
        // All three were created by GlobalDeclarationInstantiation, but the
        // script threw before initializing them, so they stay in their
        // temporal dead zone for the life of the Realm.
        for name in ["l", "c", "tdz"] {
            assert!(realm.get_global(name).is_none());
            let error = realm.get_binding_value(name).unwrap_err();
            assert_eq!(error.kind, ErrorKind::ReferenceError);
            assert_eq!(
                error.message,
                format!("Cannot access '{name}' before initialization")
            );
            assert_eq!(
                realm.set_global(name, Value::Null).unwrap_err().kind,
                ErrorKind::ReferenceError
            );
        }
        assert_eq!(
            realm.get_binding_value("nope").unwrap_err().message,
            "nope is not defined"
        );

        assert!(matches!(
            realm.evaluate("let m = 1; const k = 2; var v = 3"),
            Completion::Normal(Value::Undefined)
        ));
        realm.set_global("m", Value::Number(10.0)).unwrap();
        realm.set_global("v", Value::Number(30.0)).unwrap();
        assert!(matches!(
            realm.evaluate("m + v"),
            Completion::Normal(Value::Number(40.0))
        ));
        assert_eq!(
            realm.set_global("k", Value::Null).unwrap_err(),
            Exception::new(ErrorKind::TypeError, "Assignment to constant variable.")
        );
        assert!(matches!(realm.get_global("k"), Some(Value::Number(2.0))));

        // A host-created global is a configurable property: a later `let`
        // may shadow it, and then the host writes the lexical binding.
        realm.set_global("h", Value::Number(1.0)).unwrap();
        assert!(matches!(
            realm.evaluate("let h = 2; h"),
            Completion::Normal(Value::Number(2.0))
        ));
        realm.set_global("h", Value::Number(3.0)).unwrap();
        assert!(matches!(
            realm.evaluate("h"),
            Completion::Normal(Value::Number(3.0))
        ));
    }

    #[test]
    fn realms_are_isolated() {
        let mut a = Realm::new();
        let b = Realm::new();
        a.set_global("x", Value::Number(1.0)).unwrap();
        assert!(b.get_global("x").is_none());
    }
}
