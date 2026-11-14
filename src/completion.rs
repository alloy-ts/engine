use super::value::Value;

/// The native error constructor an abrupt completion maps to at the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    SyntaxError,
    ReferenceError,
    TypeError,
    RangeError,
}

impl ErrorKind {
    pub fn name(self) -> &'static str {
        match self {
            ErrorKind::SyntaxError => "SyntaxError",
            ErrorKind::ReferenceError => "ReferenceError",
            ErrorKind::TypeError => "TypeError",
            ErrorKind::RangeError => "RangeError",
        }
    }
}

/// A thrown error. The runtime has no objects yet, so it records the error
/// kind and message rather than a thrown Value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exception {
    pub kind: ErrorKind,
    pub message: String,
}

impl Exception {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn syntax(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::SyntaxError, message)
    }
}

/// The `[[Type]]` of a [`CompletionRecord`] (ECMA-262 §6.2.4, Table 7).
///
/// Only the types this runtime can produce exist. `break`, `continue` and
/// `return` are added together with the statements that create them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionType {
    Normal,
    Throw,
}

/// The ECMA-262 Completion Record (§6.2.4) for the completion types this
/// runtime produces.
///
/// - `[[Type]]` is the variant ([`CompletionRecord::completion_type`]).
/// - `[[Value]]` is the payload. A normal completion's value is a language
///   value or *empty* (`None`); a throw completion's value is never empty and
///   is an [`Exception`] until the runtime has error objects.
/// - `[[Target]]` must be empty unless `[[Type]]` is break or continue, so it
///   is always empty here ([`CompletionRecord::target`]).
#[derive(Clone, Debug)]
pub enum CompletionRecord {
    Normal(Option<Value>),
    Throw(Exception),
}

impl CompletionRecord {
    /// NormalCompletion(empty), the result of statements that produce no
    /// value (declarations, empty blocks).
    pub const EMPTY: Self = Self::Normal(None);

    pub fn completion_type(&self) -> CompletionType {
        match self {
            Self::Normal(_) => CompletionType::Normal,
            Self::Throw(_) => CompletionType::Throw,
        }
    }

    /// `[[Target]]`: always empty for normal and throw completions.
    pub fn target(&self) -> Option<&str> {
        None
    }

    pub fn is_abrupt(&self) -> bool {
        self.completion_type() != CompletionType::Normal
    }

    /// UpdateEmpty(completionRecord, value) (§6.2.4.4): replaces an empty
    /// `[[Value]]` with `value`, keeping `[[Type]]` and `[[Target]]`. A
    /// throw completion is never empty and is returned unchanged.
    pub fn update_empty(self, value: Option<Value>) -> Self {
        match self {
            Self::Normal(None) => Self::Normal(value),
            other => other,
        }
    }
}

impl From<Exception> for CompletionRecord {
    fn from(exception: Exception) -> Self {
        Self::Throw(exception)
    }
}

/// The result of running a Script, as an embedding sees it: the script's
/// completion value or the exception it threw.
///
/// This is the caller-facing view of the [`CompletionRecord`] that
/// ScriptEvaluation returns, which is always normal or throw and whose
/// normal value is never empty (§16.1.6 turns empty into undefined).
#[derive(Clone, Debug)]
pub enum Completion<T> {
    Normal(T),
    Throw(Exception),
}

impl From<CompletionRecord> for Completion<Value> {
    /// ScriptEvaluation step: a normal completion whose `[[Value]]` is empty
    /// becomes NormalCompletion(undefined).
    fn from(record: CompletionRecord) -> Self {
        match record {
            CompletionRecord::Normal(value) => Self::Normal(value.unwrap_or(Value::Undefined)),
            CompletionRecord::Throw(exception) => Self::Throw(exception),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn number(record: &CompletionRecord) -> Option<f64> {
        match record {
            CompletionRecord::Normal(Some(Value::Number(n))) => Some(*n),
            _ => None,
        }
    }

    #[test]
    fn update_empty_follows_the_spec() {
        // An empty normal completion takes the supplied value...
        let updated = CompletionRecord::EMPTY.update_empty(Some(Value::Number(1.0)));
        assert_eq!(number(&updated), Some(1.0));
        assert_eq!(updated.completion_type(), CompletionType::Normal);
        assert_eq!(updated.target(), None);
        // ...or stays empty when that value is empty too.
        assert!(matches!(
            CompletionRecord::EMPTY.update_empty(None),
            CompletionRecord::Normal(None)
        ));
        // A non-empty value is kept.
        let kept = CompletionRecord::Normal(Some(Value::Number(2.0)))
            .update_empty(Some(Value::Number(1.0)));
        assert_eq!(number(&kept), Some(2.0));
        // A throw completion is never empty and is returned unchanged.
        let thrown = CompletionRecord::from(Exception::new(ErrorKind::TypeError, "boom"))
            .update_empty(Some(Value::Number(1.0)));
        assert!(thrown.is_abrupt());
        assert_eq!(thrown.completion_type(), CompletionType::Throw);
        assert!(
            matches!(thrown, CompletionRecord::Throw(e) if e == Exception::new(ErrorKind::TypeError, "boom"))
        );
    }

    #[test]
    fn script_view_turns_empty_into_undefined() {
        assert!(matches!(
            Completion::from(CompletionRecord::EMPTY),
            Completion::Normal(Value::Undefined)
        ));
        assert!(matches!(
            Completion::from(CompletionRecord::Normal(Some(Value::Null))),
            Completion::Normal(Value::Null)
        ));
        assert!(matches!(
            Completion::from(CompletionRecord::from(Exception::syntax("x"))),
            Completion::Throw(_)
        ));
    }
}
