//! Script Records, the syntax tree they own, and the static semantics
//! (declared names) that declaration instantiation needs.

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
    /// `name = value` (operator `None`) or a compound assignment such as
    /// `name += value`. The target is always an IdentifierReference.
    Assign(Option<BinaryOp>, Box<str>, Box<Expr>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeclarationKind {
    Var,
    Let,
    Const,
}

/// One `name` or `name = initializer` in a declaration.
#[derive(Clone, Debug, PartialEq)]
pub struct VariableBinding {
    pub name: Box<str>,
    pub initializer: Option<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Expression(Expr),
    /// A VariableStatement (`var`) or a LexicalDeclaration (`let`/`const`).
    Declaration(DeclarationKind, Vec<VariableBinding>),
    /// A Block's StatementList.
    Block(Vec<Statement>),
}

/// The `let`/`const` declarations directly in `list` (not in nested
/// blocks): LexicallyScopedDeclarations, flattened to bound names. For a
/// Script this is also its LexicallyDeclaredNames.
pub(super) fn lexically_declared_names(
    list: &[Statement],
) -> impl Iterator<Item = (DeclarationKind, &str)> {
    list.iter()
        .filter_map(|statement| match statement {
            Statement::Declaration(kind @ (DeclarationKind::Let | DeclarationKind::Const), b) => {
                Some(b.iter().map(move |binding| (*kind, &*binding.name)))
            }
            _ => None,
        })
        .flatten()
}

/// VarDeclaredNames of `list`: `var` names, including those inside nested
/// blocks, in source order and possibly repeated.
pub(super) fn var_declared_names(list: &[Statement]) -> Vec<&str> {
    fn collect<'a>(list: &'a [Statement], names: &mut Vec<&'a str>) {
        for statement in list {
            match statement {
                Statement::Declaration(DeclarationKind::Var, bindings) => {
                    names.extend(bindings.iter().map(|binding| &*binding.name));
                }
                Statement::Block(body) => collect(body, names),
                _ => {}
            }
        }
    }
    let mut names = Vec::new();
    collect(list, &mut names);
    names
}

/// A parsed Script (ECMA-262 §16.1.4), reduced to its `[[ECMAScriptCode]]`:
/// the top-level statement list. Empty statements are dropped because they
/// complete with empty and never change the completion value.
#[derive(Clone, Debug, PartialEq)]
pub struct ScriptRecord {
    pub(super) statements: Vec<Statement>,
}

impl ScriptRecord {
    /// ParseScript: returns the record, or the first early SyntaxError
    /// (including duplicate lexical declarations within this Script).
    pub fn parse(source: &str) -> Result<Self, Exception> {
        Ok(Self {
            statements: parser::parse(source)?,
        })
    }

    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }
}
