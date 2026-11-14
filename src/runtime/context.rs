use super::completion::Completion;
use super::realm::Realm;
use super::value::Value;

/// Returned by every operation on a disposed [`RuntimeContext`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Disposed;

/// Lifecycle wrapper around one Realm. Disposal drops the Realm and its heap;
/// afterwards every operation fails with [`Disposed`].
#[derive(Debug)]
pub struct RuntimeContext {
    realm: Option<Realm>,
}

impl Default for RuntimeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeContext {
    pub fn new() -> Self {
        Self {
            realm: Some(Realm::new()),
        }
    }

    pub fn is_disposed(&self) -> bool {
        self.realm.is_none()
    }

    /// Releases the Realm. Idempotent.
    pub fn dispose(&mut self) {
        self.realm = None;
    }

    pub fn realm(&self) -> Result<&Realm, Disposed> {
        self.realm.as_ref().ok_or(Disposed)
    }

    pub fn realm_mut(&mut self) -> Result<&mut Realm, Disposed> {
        self.realm.as_mut().ok_or(Disposed)
    }

    pub fn evaluate(&mut self, source: &str) -> Result<Completion<Value>, Disposed> {
        Ok(self.realm_mut()?.evaluate(source))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn globals_persist_until_disposal() {
        let mut context = RuntimeContext::new();
        context
            .realm_mut()
            .unwrap()
            .set_global("x", Value::Number(2.0))
            .unwrap();
        assert!(matches!(
            context.evaluate("x * 21"),
            Ok(Completion::Normal(Value::Number(42.0)))
        ));
        assert!(!context.is_disposed());
        context.dispose();
        context.dispose();
        assert!(context.is_disposed());
        assert!(matches!(context.evaluate("1"), Err(Disposed)));
        assert!(context.realm().is_err());
        assert!(context.realm_mut().is_err());
    }

    #[test]
    fn contexts_are_isolated() {
        let mut a = RuntimeContext::new();
        let mut b = RuntimeContext::new();
        a.realm_mut()
            .unwrap()
            .set_global("shared", Value::Boolean(true))
            .unwrap();
        assert!(matches!(
            a.evaluate("shared"),
            Ok(Completion::Normal(Value::Boolean(true)))
        ));
        assert!(matches!(b.evaluate("shared"), Ok(Completion::Throw(_))));
        a.dispose();
        assert!(matches!(
            b.evaluate("1 + 1"),
            Ok(Completion::Normal(Value::Number(2.0)))
        ));
    }
}
