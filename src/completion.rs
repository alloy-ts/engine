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

/// Minimal stand-in for the ECMA-262 Completion Record: only `normal` and
/// `throw` exist because the MVP has no break/continue/return or try/finally.
#[derive(Clone, Debug)]
pub enum Completion<T> {
    Normal(T),
    Throw(Exception),
}
