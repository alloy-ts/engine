//! Environment Records (ECMA-262 §9.1): the Declarative Environment Record
//! and the Global Environment Record. Object Environment Records exist only
//! as the global record's object part, because the runtime has no objects.

use std::collections::HashMap;

use super::completion::Exception;
use super::value::Value;

fn not_initialized(name: &str) -> Exception {
    Exception::reference(format!("Cannot access '{name}' before initialization"))
}

pub(crate) fn not_defined(name: &str) -> Exception {
    Exception::reference(format!("{name} is not defined"))
}

#[derive(Clone, Copy, Debug)]
struct Binding {
    /// `None` while the binding is uninitialized (its temporal dead zone).
    value: Option<Value>,
    mutable: bool,
    /// A strict binding makes every SetMutableBinding on it strict.
    strict: bool,
}

/// Declarative Environment Record (§9.1.1.1).
///
/// The `deletable` flag of CreateMutableBinding is not stored: nothing can
/// delete a binding yet. `[[OuterEnv]]` is kept by the owner of the record
/// (see the evaluator's scope stack), not in the record itself.
#[derive(Debug, Default)]
pub(crate) struct DeclarativeEnvironment {
    bindings: HashMap<Box<str>, Binding>,
}

impl DeclarativeEnvironment {
    pub fn has_binding(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
    }

    /// CreateMutableBinding(N, D): a new uninitialized mutable binding.
    pub fn create_mutable_binding(&mut self, name: &str) {
        debug_assert!(!self.has_binding(name));
        self.bindings.insert(
            name.into(),
            Binding {
                value: None,
                mutable: true,
                strict: false,
            },
        );
    }

    /// CreateImmutableBinding(N, S): a new uninitialized immutable binding.
    pub fn create_immutable_binding(&mut self, name: &str, strict: bool) {
        debug_assert!(!self.has_binding(name));
        self.bindings.insert(
            name.into(),
            Binding {
                value: None,
                mutable: false,
                strict,
            },
        );
    }

    /// InitializeBinding(N, V): ends the binding's temporal dead zone.
    pub fn initialize_binding(&mut self, name: &str, value: Value) {
        let binding = self
            .bindings
            .get_mut(name)
            .expect("InitializeBinding on a missing binding");
        debug_assert!(binding.value.is_none());
        binding.value = Some(value);
    }

    /// SetMutableBinding(N, V, S).
    pub fn set_mutable_binding(
        &mut self,
        name: &str,
        value: Value,
        strict: bool,
    ) -> Result<(), Exception> {
        let Some(binding) = self.bindings.get_mut(name) else {
            // Only reachable once bindings can be deleted (spec step 1).
            if strict {
                return Err(not_defined(name));
            }
            self.create_mutable_binding(name);
            self.initialize_binding(name, value);
            return Ok(());
        };
        let strict = strict || binding.strict;
        if binding.value.is_none() {
            Err(not_initialized(name))
        } else if binding.mutable {
            binding.value = Some(value);
            Ok(())
        } else if strict {
            Err(Exception::type_error("Assignment to constant variable."))
        } else {
            Ok(())
        }
    }

    /// GetBindingValue(N, S). Throws a ReferenceError in the temporal dead
    /// zone regardless of strictness.
    pub fn get_binding_value(&self, name: &str) -> Result<Value, Exception> {
        let binding = self
            .bindings
            .get(name)
            .expect("GetBindingValue on a missing binding");
        binding.value.ok_or_else(|| not_initialized(name))
    }
}

/// An own data property of the global object. `[[Enumerable]]` is not
/// modelled because nothing can enumerate the global object yet.
#[derive(Clone, Copy, Debug)]
struct Property {
    value: Value,
    writable: bool,
    configurable: bool,
}

/// The global object, reduced to a map of own data properties.
///
/// Deviation: the runtime has no objects, so there is no prototype chain
/// (HasProperty is HasOwnProperty), no accessors, and the object is always
/// extensible. It is not reachable as a value (`globalThis`, `this`).
#[derive(Debug, Default)]
struct GlobalObject {
    properties: HashMap<Box<str>, Property>,
}

impl GlobalObject {
    fn has_property(&self, name: &str) -> bool {
        self.properties.contains_key(name)
    }

    /// OrdinarySet for a data property on an extensible object with no
    /// prototype: updates a writable property, creates a missing one as
    /// writable and configurable, and reports failure for a read-only one.
    fn set(&mut self, name: &str, value: Value) -> bool {
        match self.properties.get_mut(name) {
            Some(property) if !property.writable => false,
            Some(property) => {
                property.value = value;
                true
            }
            None => {
                self.properties.insert(
                    name.into(),
                    Property {
                        value,
                        writable: true,
                        configurable: true,
                    },
                );
                true
            }
        }
    }
}

/// Global Environment Record (§9.1.1.4): an object record over the global
/// object plus a declarative record for top-level `let` / `const`.
///
/// `[[GlobalThisValue]]` is omitted because the runtime has no objects.
#[derive(Debug)]
pub(crate) struct GlobalEnvironment {
    object_record: GlobalObject,
    pub declarative_record: DeclarativeEnvironment,
}

impl GlobalEnvironment {
    /// A global environment whose global object has the non-writable,
    /// non-configurable value properties the runtime supports (§19.1).
    pub fn new() -> Self {
        let mut object_record = GlobalObject::default();
        for (name, value) in [
            ("undefined", Value::Undefined),
            ("NaN", Value::Number(f64::NAN)),
            ("Infinity", Value::Number(f64::INFINITY)),
        ] {
            object_record.properties.insert(
                name.into(),
                Property {
                    value,
                    writable: false,
                    configurable: false,
                },
            );
        }
        Self {
            object_record,
            declarative_record: DeclarativeEnvironment::default(),
        }
    }

    /// HasBinding(N).
    pub fn has_binding(&self, name: &str) -> bool {
        self.declarative_record.has_binding(name) || self.object_record.has_property(name)
    }

    /// HasLexicalDeclaration(envRecord, N).
    pub fn has_lexical_declaration(&self, name: &str) -> bool {
        self.declarative_record.has_binding(name)
    }

    /// HasRestrictedGlobalProperty(envRecord, N): a non-configurable global
    /// object property (a `var` or a built-in) that a lexical declaration
    /// must not shadow.
    pub fn has_restricted_global_property(&self, name: &str) -> bool {
        self.object_record
            .properties
            .get(name)
            .is_some_and(|property| !property.configurable)
    }

    /// CreateGlobalVarBinding(envRecord, N, D) with D = false (Script var
    /// declarations are not deletable). An existing property is reused.
    /// CanDeclareGlobalVar is always true because the global object is
    /// always extensible.
    pub fn create_global_var_binding(&mut self, name: &str) {
        self.object_record
            .properties
            .entry(name.into())
            .or_insert(Property {
                value: Value::Undefined,
                writable: true,
                configurable: false,
            });
    }

    /// GetBindingValue(N, S).
    pub fn get_binding_value(&self, name: &str, strict: bool) -> Result<Value, Exception> {
        if self.declarative_record.has_binding(name) {
            return self.declarative_record.get_binding_value(name);
        }
        match self.object_record.properties.get(name) {
            Some(property) => Ok(property.value),
            None if strict => Err(not_defined(name)),
            None => Ok(Value::Undefined),
        }
    }

    /// SetMutableBinding(N, V, S). In the object part a missing binding is
    /// a ReferenceError only when strict; otherwise Set creates it, and a
    /// read-only property is a TypeError when strict and ignored otherwise.
    pub fn set_mutable_binding(
        &mut self,
        name: &str,
        value: Value,
        strict: bool,
    ) -> Result<(), Exception> {
        if self.declarative_record.has_binding(name) {
            return self
                .declarative_record
                .set_mutable_binding(name, value, strict);
        }
        if strict && !self.object_record.has_property(name) {
            return Err(not_defined(name));
        }
        self.set_global_object_property(name, value, strict)
    }

    /// Set(globalObject, N, V, Throw): the step PutValue performs for an
    /// unresolvable reference in sloppy code, creating the property.
    pub fn set_global_object_property(
        &mut self,
        name: &str,
        value: Value,
        throw: bool,
    ) -> Result<(), Exception> {
        if !self.object_record.set(name, value) && throw {
            return Err(Exception::type_error(format!(
                "Cannot assign to read only global '{name}'"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrorKind;

    fn number(result: Result<Value, Exception>) -> f64 {
        match result {
            Ok(Value::Number(n)) => n,
            other => panic!("expected a number, got {other:?}"),
        }
    }

    #[test]
    fn declarative_bindings_have_a_temporal_dead_zone() {
        let mut env = DeclarativeEnvironment::default();
        env.create_mutable_binding("x");
        assert!(env.has_binding("x"));
        let error = env.get_binding_value("x").unwrap_err();
        assert_eq!(error.kind, ErrorKind::ReferenceError);
        assert_eq!(error.message, "Cannot access 'x' before initialization");
        // Assignment in the TDZ is a ReferenceError even when sloppy.
        let error = env
            .set_mutable_binding("x", Value::Null, false)
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::ReferenceError);
        env.initialize_binding("x", Value::Number(1.0));
        env.set_mutable_binding("x", Value::Number(2.0), false)
            .unwrap();
        assert_eq!(number(env.get_binding_value("x")), 2.0);
    }

    #[test]
    fn strict_immutable_bindings_reject_assignment() {
        let mut env = DeclarativeEnvironment::default();
        env.create_immutable_binding("c", true);
        env.initialize_binding("c", Value::Number(1.0));
        // A strict binding throws even from sloppy code.
        let error = env
            .set_mutable_binding("c", Value::Number(2.0), false)
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::TypeError);
        assert_eq!(error.message, "Assignment to constant variable.");
        // A non-strict immutable binding ignores sloppy writes.
        env.create_immutable_binding("n", false);
        env.initialize_binding("n", Value::Number(1.0));
        env.set_mutable_binding("n", Value::Number(2.0), false)
            .unwrap();
        assert_eq!(number(env.get_binding_value("n")), 1.0);
        assert_eq!(
            env.set_mutable_binding("n", Value::Number(2.0), true)
                .unwrap_err()
                .kind,
            ErrorKind::TypeError
        );
    }

    #[test]
    fn global_record_splits_object_and_declarative_parts() {
        let mut env = GlobalEnvironment::new();
        assert!(env.has_restricted_global_property("undefined"));
        assert!(!env.has_lexical_declaration("undefined"));

        env.create_global_var_binding("v");
        assert!(env.has_restricted_global_property("v"));
        assert!(matches!(
            env.get_binding_value("v", true),
            Ok(Value::Undefined)
        ));

        // Implicit globals are configurable, so not restricted.
        env.set_global_object_property("implicit", Value::Null, false)
            .unwrap();
        assert!(env.has_binding("implicit"));
        assert!(!env.has_restricted_global_property("implicit"));

        // A lexical binding shadows the object part.
        env.declarative_record.create_mutable_binding("implicit");
        assert!(env.has_lexical_declaration("implicit"));
        assert_eq!(
            env.get_binding_value("implicit", false).unwrap_err().kind,
            ErrorKind::ReferenceError
        );

        // Missing bindings: ReferenceError when strict, created when sloppy.
        assert_eq!(
            env.set_mutable_binding("missing", Value::Null, true)
                .unwrap_err()
                .kind,
            ErrorKind::ReferenceError
        );
        assert!(matches!(
            env.get_binding_value("missing", false),
            Ok(Value::Undefined)
        ));
        env.set_mutable_binding("missing", Value::Number(3.0), false)
            .unwrap();
        assert_eq!(number(env.get_binding_value("missing", true)), 3.0);

        // Read-only properties: ignored when sloppy, TypeError when strict.
        env.set_mutable_binding("NaN", Value::Number(1.0), false)
            .unwrap();
        assert!(number(env.get_binding_value("NaN", true)).is_nan());
        assert_eq!(
            env.set_mutable_binding("NaN", Value::Number(1.0), true)
                .unwrap_err()
                .kind,
            ErrorKind::TypeError
        );
    }
}
