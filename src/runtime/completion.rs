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

    pub(crate) fn reference(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ReferenceError, message)
    }

    pub(crate) fn type_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::TypeError, message)
    }
}

/// The `[[Type]]` of a [`CompletionRecord`] (ECMA-262 §6.2.4, Table 7).
///
/// Only the types this runtime can produce exist. `return` is added with
/// functions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionType {
    Normal,
    Break,
    Continue,
    Throw,
}

/// The ECMA-262 Completion Record (§6.2.4) for the completion types this
/// runtime produces.
///
/// - `[[Type]]` is the variant ([`CompletionRecord::completion_type`]).
/// - `[[Value]]` is the payload. A normal completion's value is a language
///   value or *empty* (`None`); a throw completion carries the thrown Value.
/// - `[[Target]]` is present only for labelled break/continue and is consumed
///   by the matching labelled statement or iteration statement.
#[derive(Clone, Debug)]
pub enum CompletionRecord {
    Normal(Option<Value>),
    Break {
        value: Option<Value>,
        target: Option<Box<str>>,
    },
    Continue {
        value: Option<Value>,
        target: Option<Box<str>>,
    },
    Throw(Value),
}

impl CompletionRecord {
    /// NormalCompletion(empty), the result of statements that produce no
    /// value (declarations, empty blocks).
    pub const EMPTY: Self = Self::Normal(None);

    pub fn completion_type(&self) -> CompletionType {
        match self {
            Self::Normal(_) => CompletionType::Normal,
            Self::Break { .. } => CompletionType::Break,
            Self::Continue { .. } => CompletionType::Continue,
            Self::Throw(_) => CompletionType::Throw,
        }
    }

    pub fn target(&self) -> Option<&str> {
        match self {
            Self::Break { target, .. } | Self::Continue { target, .. } => target.as_deref(),
            Self::Normal(_) | Self::Throw(_) => None,
        }
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
            Self::Break {
                value: None,
                target,
            } => Self::Break { value, target },
            Self::Continue {
                value: None,
                target,
            } => Self::Continue { value, target },
            other => other,
        }
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
    Throw(T),
}

impl From<CompletionRecord> for Completion<Value> {
    /// ScriptEvaluation step: a normal completion whose `[[Value]]` is empty
    /// becomes NormalCompletion(undefined).
    fn from(record: CompletionRecord) -> Self {
        match record {
            CompletionRecord::Normal(value) => Self::Normal(value.unwrap_or(Value::Undefined)),
            CompletionRecord::Throw(value) => Self::Throw(value),
            CompletionRecord::Break { .. } | CompletionRecord::Continue { .. } => {
                unreachable!("loop control cannot escape ScriptEvaluation")
            }
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
        let thrown = CompletionRecord::Throw(Value::Null).update_empty(Some(Value::Number(1.0)));
        assert!(thrown.is_abrupt());
        assert_eq!(thrown.completion_type(), CompletionType::Throw);
        assert!(matches!(thrown, CompletionRecord::Throw(Value::Null)));

        let broken = CompletionRecord::Break {
            value: None,
            target: Some("outer".into()),
        }
        .update_empty(Some(Value::Number(3.0)));
        assert_eq!(broken.completion_type(), CompletionType::Break);
        assert_eq!(broken.target(), Some("outer"));
        assert!(matches!(
            broken,
            CompletionRecord::Break {
                value: Some(Value::Number(3.0)),
                ..
            }
        ));

        let continued = CompletionRecord::Continue {
            value: None,
            target: None,
        }
        .update_empty(Some(Value::Number(4.0)));
        assert_eq!(continued.completion_type(), CompletionType::Continue);
        assert_eq!(continued.target(), None);
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
            Completion::from(CompletionRecord::Throw(Value::Boolean(true))),
            Completion::Throw(Value::Boolean(true))
        ));
    }
}
