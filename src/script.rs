//! Script Records and the expression syntax tree they own.

use super::completion::Exception;
use super::parser;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Mul,
    Div,
    Rem,
    Add,
    Sub,
    Lt,
    Gt,
    LtEq,
    GtEq,
    LooseEq,
    LooseNe,
    StrictEq,
    StrictNe,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Number(f64),
    String(Box<str>),
    Boolean(bool),
    Null,
    Identifier(Box<str>),
    Unary(UnaryOp, Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
}

/// A parsed Script (ECMA-262 §16.1.4), reduced to what the MVP needs: the
/// ordered expression statements. Empty statements are dropped because they
/// never change the completion value.
#[derive(Clone, Debug, PartialEq)]
pub struct ScriptRecord {
    pub(super) statements: Vec<Expr>,
}

impl ScriptRecord {
    /// ParseScript: returns the record, or the first early SyntaxError.
    pub fn parse(source: &str) -> Result<Self, Exception> {
        Ok(Self {
            statements: parser::parse(source)?,
        })
    }

    pub fn statements(&self) -> &[Expr] {
        &self.statements
    }
}
