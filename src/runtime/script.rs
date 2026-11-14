//! Script Records, the syntax tree they own, and the static semantics
//! (declared names) that declaration instantiation needs.

use super::completion::Exception;
use super::parser;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
    Typeof,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    LogicalOr,
    LogicalAnd,
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
pub struct CatchClause {
    pub binding: Option<Box<str>>,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Empty,
    Expression(Expr),
    /// A VariableStatement (`var`) or a LexicalDeclaration (`let`/`const`).
    Declaration(DeclarationKind, Vec<VariableBinding>),
    /// A Block's StatementList.
    Block(Vec<Statement>),
    If {
        condition: Expr,
        consequent: Box<Statement>,
        alternate: Option<Box<Statement>>,
    },
    While {
        condition: Expr,
        body: Box<Statement>,
    },
    DoWhile {
        body: Box<Statement>,
        condition: Expr,
    },
    Labelled {
        label: Box<str>,
        body: Box<Statement>,
    },
    Break(Option<Box<str>>),
    Continue(Option<Box<str>>),
    Throw(Expr),
    Try {
        body: Vec<Statement>,
        catch: Option<CatchClause>,
        finally: Option<Vec<Statement>>,
    },
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
    fn collect_statement<'a>(statement: &'a Statement, names: &mut Vec<&'a str>) {
        match statement {
            Statement::Declaration(DeclarationKind::Var, bindings) => {
                names.extend(bindings.iter().map(|binding| &*binding.name));
            }
            Statement::Block(body) => collect(body, names),
            Statement::If {
                consequent,
                alternate,
                ..
            } => {
                collect_statement(consequent, names);
                if let Some(alternate) = alternate {
                    collect_statement(alternate, names);
                }
            }
            Statement::While { body, .. } => collect_statement(body, names),
            Statement::DoWhile { body, .. } => collect_statement(body, names),
            Statement::Labelled { body, .. } => collect_statement(body, names),
            Statement::Try {
                body,
                catch,
                finally,
            } => {
                collect(body, names);
                if let Some(catch) = catch {
                    collect(&catch.body, names);
                }
                if let Some(finally) = finally {
                    collect(finally, names);
                }
            }
            _ => {}
        }
    }
    fn collect<'a>(list: &'a [Statement], names: &mut Vec<&'a str>) {
        for statement in list {
            collect_statement(statement, names);
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
