//! Recursive-descent parser for the supported Script grammar: statement
//! lists of expression statements, `var` / `let` / `const` declarations and
//! blocks. Expressions follow ECMA-262 precedence (loosest first):
//! assignment, equality, relational, additive, multiplicative, unary,
//! primary. Static-semantics early errors run after parsing.

use std::collections::HashSet;

use super::completion::{ErrorKind, Exception};
use super::lexer::{Token, TokenKind, tokenize};
use super::script::{
    BinaryOp, CatchClause, DeclarationKind, Expr, Statement, UnaryOp, VariableBinding,
    lexically_declared_names, var_declared_names,
};

/// Maximum syntax-tree depth. Parsing, evaluation and dropping the tree all
/// recurse over it, so the bound keeps hostile input from overflowing the
/// native stack. Blocks, parentheses, unary operators and assignment
/// right-hand sides share the parser's recursion budget.
pub const MAX_DEPTH: usize = 256;

pub fn parse(source: &str) -> Result<Vec<Statement>, Exception> {
    let tokens = tokenize(source)?;
    let mut parser = Parser {
        tokens,
        pos: 0,
        depth: 0,
        loop_depth: 0,
        labels: Vec::new(),
    };
    let statements = parser.statement_list(TokenKind::Eof)?;
    check_declarations(&statements)?;
    Ok(statements)
}

pub(super) fn redeclaration(name: &str) -> Exception {
    Exception::syntax(format!("Identifier '{name}' has already been declared"))
}

/// The declaration early errors of a Script (§16.1.1) or Block (§14.2.1),
/// applied to `list` and every nested block: LexicallyDeclaredNames has no
/// duplicates and does not intersect VarDeclaredNames.
fn check_declarations(list: &[Statement]) -> Result<(), Exception> {
    let mut lexical = HashSet::new();
    for (_, name) in lexically_declared_names(list) {
        if !lexical.insert(name) {
            return Err(redeclaration(name));
        }
    }
    if let Some(name) = var_declared_names(list)
        .into_iter()
        .find(|name| lexical.contains(name))
    {
        return Err(redeclaration(name));
    }
    fn check_nested(statement: &Statement) -> Result<(), Exception> {
        match statement {
            Statement::Block(body) => check_declarations(body),
            Statement::If {
                consequent,
                alternate,
                ..
            } => {
                check_nested(consequent)?;
                if let Some(alternate) = alternate {
                    check_nested(alternate)?;
                }
                Ok(())
            }
            Statement::While { body, .. } => check_nested(body),
            Statement::Labelled { body, .. } => check_nested(body),
            Statement::Try {
                body,
                catch,
                finally,
            } => {
                check_declarations(body)?;
                if let Some(catch) = catch {
                    check_declarations(&catch.body)?;
                    if let Some(binding) = &catch.binding
                        && lexically_declared_names(&catch.body)
                            .any(|(_, name)| name == &**binding)
                    {
                        return Err(redeclaration(binding));
                    }
                }
                if let Some(finally) = finally {
                    check_declarations(finally)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    for statement in list {
        check_nested(statement)?;
    }
    Ok(())
}

fn too_deep() -> Exception {
    Exception::new(
        ErrorKind::RangeError,
        format!("Expression nesting exceeds the limit of {MAX_DEPTH}"),
    )
}

fn describe(kind: &TokenKind) -> String {
    let text = match kind {
        TokenKind::Eof => return "Unexpected end of input".to_owned(),
        TokenKind::Number(_) => return "Unexpected number".to_owned(),
        TokenKind::String(_) => return "Unexpected string".to_owned(),
        TokenKind::Identifier(name) => return format!("Unexpected identifier '{name}'"),
        TokenKind::True => "true",
        TokenKind::False => "false",
        TokenKind::Null => "null",
        TokenKind::Var => "var",
        TokenKind::Const => "const",
        TokenKind::Typeof => "typeof",
        TokenKind::If => "if",
        TokenKind::Else => "else",
        TokenKind::While => "while",
        TokenKind::Break => "break",
        TokenKind::Continue => "continue",
        TokenKind::Throw => "throw",
        TokenKind::Try => "try",
        TokenKind::Catch => "catch",
        TokenKind::Finally => "finally",
        TokenKind::Plus => "+",
        TokenKind::Minus => "-",
        TokenKind::Star => "*",
        TokenKind::Slash => "/",
        TokenKind::Percent => "%",
        TokenKind::Bang => "!",
        TokenKind::Lt => "<",
        TokenKind::LtEq => "<=",
        TokenKind::Gt => ">",
        TokenKind::GtEq => ">=",
        TokenKind::EqEq => "==",
        TokenKind::NotEq => "!=",
        TokenKind::EqEqEq => "===",
        TokenKind::NotEqEq => "!==",
        TokenKind::AndAnd => "&&",
        TokenKind::OrOr => "||",
        TokenKind::LParen => "(",
        TokenKind::RParen => ")",
        TokenKind::LBrace => "{",
        TokenKind::RBrace => "}",
        TokenKind::Comma => ",",
        TokenKind::Semicolon => ";",
        TokenKind::Colon => ":",
        TokenKind::Assign => "=",
        TokenKind::CompoundAssign(op) => match op {
            BinaryOp::Add => "+=",
            BinaryOp::Sub => "-=",
            BinaryOp::Mul => "*=",
            BinaryOp::Div => "/=",
            _ => "%=",
        },
    };
    format!("Unexpected token '{text}'")
}

/// A subtree paired with its depth.
type Node = (Expr, usize);

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    /// Current recursion depth through unary operators and parentheses.
    depth: usize,
    /// Iteration statements containing the statement currently being parsed.
    loop_depth: usize,
    /// Active labels and whether each ultimately labels an iteration.
    labels: Vec<(Box<str>, bool)>,
}

impl Parser {
    fn peek(&self) -> &Token {
        self.peek_at(0)
    }

    fn peek_at(&self, offset: usize) -> &Token {
        // `tokenize` always ends with Eof and `pos` never advances past it.
        &self.tokens[(self.pos + offset).min(self.tokens.len() - 1)]
    }

    /// Statements up to (and consuming) `end`: `}` for a Block, end of input
    /// for a Script.
    fn statement_list(&mut self, end: TokenKind) -> Result<Vec<Statement>, Exception> {
        let mut statements = Vec::new();
        loop {
            match &self.peek().kind {
                kind if *kind == end => {
                    self.pos += 1;
                    return Ok(statements);
                }
                TokenKind::Eof | TokenKind::RBrace => return Err(self.unexpected()),
                TokenKind::Semicolon => self.pos += 1,
                _ => statements.push(self.statement()?),
            }
        }
    }

    fn statement(&mut self) -> Result<Statement, Exception> {
        let kind = match &self.peek().kind {
            TokenKind::Semicolon => {
                self.pos += 1;
                return Ok(Statement::Empty);
            }
            TokenKind::If => return self.if_statement(),
            TokenKind::While => return self.while_statement(),
            TokenKind::Break => return self.loop_control(false),
            TokenKind::Continue => return self.loop_control(true),
            TokenKind::Throw => return self.throw_statement(),
            TokenKind::Try => return self.try_statement(),
            TokenKind::LBrace => {
                self.pos += 1;
                let body = self.nested(|parser| parser.statement_list(TokenKind::RBrace))?;
                return Ok(Statement::Block(body));
            }
            TokenKind::Var => DeclarationKind::Var,
            TokenKind::Const => DeclarationKind::Const,
            TokenKind::Identifier(_)
                if matches!(self.peek_at(1).kind, TokenKind::Colon) =>
            {
                return self.labelled_statement();
            }
            // `let` is a contextual keyword: followed by a binding identifier
            // it starts a LexicalDeclaration, otherwise it is an identifier.
            TokenKind::Identifier(word)
                if &**word == "let" && matches!(self.peek_at(1).kind, TokenKind::Identifier(_)) =>
            {
                DeclarationKind::Let
            }
            _ => {
                let expression = self.expression()?.0;
                self.end_statement()?;
                return Ok(Statement::Expression(expression));
            }
        };
        self.pos += 1;
        let bindings = self.binding_list(kind)?;
        self.end_statement()?;
        Ok(Statement::Declaration(kind, bindings))
    }

    fn loop_control(&mut self, is_continue: bool) -> Result<Statement, Exception> {
        self.pos += 1;
        let target = if !self.peek().newline_before {
            if let TokenKind::Identifier(name) = &self.peek().kind {
                let name = name.clone();
                self.pos += 1;
                Some(name)
            } else {
                None
            }
        } else {
            None
        };

        match target.as_deref() {
            Some(label) => match self.labels.iter().rev().find(|(name, _)| &**name == label) {
                None => return Err(Exception::syntax(format!("Undefined label '{label}'"))),
                Some((_, false)) if is_continue => {
                    return Err(Exception::syntax(format!(
                        "Illegal continue statement: '{label}' does not denote an iteration statement"
                    )));
                }
                Some(_) => {}
            },
            None if self.loop_depth == 0 => {
                let kind = if is_continue { "continue" } else { "break" };
                return Err(Exception::syntax(format!("Illegal {kind} statement")));
            }
            None => {}
        }

        self.end_statement()?;
        Ok(if is_continue {
            Statement::Continue(target)
        } else {
            Statement::Break(target)
        })
    }

    fn throw_statement(&mut self) -> Result<Statement, Exception> {
        self.pos += 1;
        if self.peek().newline_before {
            return Err(Exception::syntax("Illegal newline after throw"));
        }
        let value = self.expression()?.0;
        self.end_statement()?;
        Ok(Statement::Throw(value))
    }

    fn try_statement(&mut self) -> Result<Statement, Exception> {
        self.pos += 1;
        let body = self.block_body()?;
        let catch = if self.peek().kind == TokenKind::Catch {
            self.pos += 1;
            let binding = if self.peek().kind == TokenKind::LParen {
                self.pos += 1;
                let TokenKind::Identifier(name) = &self.peek().kind else {
                    return Err(self.unexpected());
                };
                let name = name.clone();
                self.pos += 1;
                if self.peek().kind != TokenKind::RParen {
                    return Err(self.unexpected());
                }
                self.pos += 1;
                Some(name)
            } else {
                None
            };
            Some(CatchClause {
                binding,
                body: self.block_body()?,
            })
        } else {
            None
        };
        let finally = if self.peek().kind == TokenKind::Finally {
            self.pos += 1;
            Some(self.block_body()?)
        } else {
            None
        };
        if catch.is_none() && finally.is_none() {
            return Err(Exception::syntax("Missing catch or finally after try"));
        }
        Ok(Statement::Try {
            body,
            catch,
            finally,
        })
    }

    fn block_body(&mut self) -> Result<Vec<Statement>, Exception> {
        if self.peek().kind != TokenKind::LBrace {
            return Err(self.unexpected());
        }
        self.pos += 1;
        self.nested(|parser| parser.statement_list(TokenKind::RBrace))
    }

    fn labelled_statement(&mut self) -> Result<Statement, Exception> {
        let TokenKind::Identifier(label) = &self.peek().kind else {
            unreachable!()
        };
        let label = label.clone();
        if self.labels.iter().any(|(active, _)| active == &label) {
            return Err(Exception::syntax(format!(
                "Label '{}' has already been declared",
                label
            )));
        }
        self.pos += 2;
        let iteration = self.labelled_item_is_iteration();
        self.labels.push((label.clone(), iteration));
        let body = self.nested(Self::statement);
        self.labels.pop();
        let body = body?;
        if matches!(
            body,
            Statement::Declaration(DeclarationKind::Let | DeclarationKind::Const, _)
        ) {
            return Err(Exception::syntax(
                "Lexical declaration cannot appear in a single-statement context",
            ));
        }
        Ok(Statement::Labelled {
            label,
            body: Box::new(body),
        })
    }

    fn labelled_item_is_iteration(&self) -> bool {
        let mut offset = 0;
        while matches!(self.peek_at(offset).kind, TokenKind::Identifier(_))
            && matches!(self.peek_at(offset + 1).kind, TokenKind::Colon)
        {
            offset += 2;
        }
        self.peek_at(offset).kind == TokenKind::While
    }

    fn if_statement(&mut self) -> Result<Statement, Exception> {
        self.pos += 1;
        if self.peek().kind != TokenKind::LParen {
            return Err(self.unexpected());
        }
        self.pos += 1;
        let condition = self.nested(Self::expression)?.0;
        if self.peek().kind != TokenKind::RParen {
            return Err(self.unexpected());
        }
        self.pos += 1;
        let consequent = self.nested(Self::statement)?;
        if matches!(
            consequent,
            Statement::Declaration(DeclarationKind::Let | DeclarationKind::Const, _)
        ) {
            return Err(Exception::syntax(
                "Lexical declaration cannot appear in a single-statement context",
            ));
        }
        let alternate = if self.peek().kind == TokenKind::Else {
            self.pos += 1;
            let statement = self.nested(Self::statement)?;
            if matches!(
                statement,
                Statement::Declaration(DeclarationKind::Let | DeclarationKind::Const, _)
            ) {
                return Err(Exception::syntax(
                    "Lexical declaration cannot appear in a single-statement context",
                ));
            }
            Some(Box::new(statement))
        } else {
            None
        };
        Ok(Statement::If {
            condition,
            consequent: Box::new(consequent),
            alternate,
        })
    }

    fn while_statement(&mut self) -> Result<Statement, Exception> {
        self.pos += 1;
        if self.peek().kind != TokenKind::LParen {
            return Err(self.unexpected());
        }
        self.pos += 1;
        let condition = self.nested(Self::expression)?.0;
        if self.peek().kind != TokenKind::RParen {
            return Err(self.unexpected());
        }
        self.pos += 1;
        self.loop_depth += 1;
        let body = self.nested(Self::statement);
        self.loop_depth -= 1;
        let body = body?;
        if matches!(
            body,
            Statement::Declaration(DeclarationKind::Let | DeclarationKind::Const, _)
        ) {
            return Err(Exception::syntax(
                "Lexical declaration cannot appear in a single-statement context",
            ));
        }
        Ok(Statement::While {
            condition,
            body: Box::new(body),
        })
    }

    /// A VariableDeclarationList or BindingList, with the LexicalDeclaration
    /// early errors (§14.3.1.1) that concern a single declaration.
    fn binding_list(&mut self, kind: DeclarationKind) -> Result<Vec<VariableBinding>, Exception> {
        let mut bindings = Vec::new();
        loop {
            let TokenKind::Identifier(name) = &self.peek().kind else {
                return Err(self.unexpected());
            };
            let name = name.clone();
            self.pos += 1;
            if kind != DeclarationKind::Var && &*name == "let" {
                return Err(Exception::syntax(
                    "let is disallowed as a lexically bound name",
                ));
            }
            let initializer = if self.peek().kind == TokenKind::Assign {
                self.pos += 1;
                Some(self.assignment()?.0)
            } else if kind == DeclarationKind::Const {
                return Err(Exception::syntax(
                    "Missing initializer in const declaration",
                ));
            } else {
                None
            };
            bindings.push(VariableBinding { name, initializer });
            if self.peek().kind != TokenKind::Comma {
                return Ok(bindings);
            }
            self.pos += 1;
        }
    }

    fn unexpected(&self) -> Exception {
        Exception::syntax(describe(&self.peek().kind))
    }

    /// A statement ends at `;`, or by automatic semicolon insertion
    /// (ECMA-262 §12.10) at end of input, before `}`, or before a token on a
    /// new line.
    fn end_statement(&mut self) -> Result<(), Exception> {
        let token = self.peek();
        match token.kind {
            TokenKind::Semicolon => {
                self.pos += 1;
                Ok(())
            }
            TokenKind::Eof | TokenKind::RBrace => Ok(()),
            // Without call expressions, `(` after an operand is never valid,
            // and ASI does not apply to it in ECMAScript either.
            TokenKind::LParen => Err(Exception::syntax("Call expressions are not supported")),
            _ if token.newline_before => Ok(()),
            _ => Err(self.unexpected()),
        }
    }

    fn expression(&mut self) -> Result<Node, Exception> {
        self.assignment()
    }

    /// AssignmentExpression: right-associative `=` and compound assignment.
    /// The target must be an identifier (optionally parenthesized), else it
    /// is an early SyntaxError.
    fn assignment(&mut self) -> Result<Node, Exception> {
        let (target, depth) = self.logical_or()?;
        let op = match self.peek().kind {
            TokenKind::Assign => None,
            TokenKind::CompoundAssign(op) => Some(op),
            _ => return Ok((target, depth)),
        };
        let Expr::Identifier(name) = target else {
            return Err(Exception::syntax("Invalid left-hand side in assignment"));
        };
        self.pos += 1;
        let (value, value_depth) = self.nested(Self::assignment)?;
        if value_depth + 1 > MAX_DEPTH {
            return Err(too_deep());
        }
        Ok((Expr::Assign(op, name, Box::new(value)), value_depth + 1))
    }

    fn logical_or(&mut self) -> Result<Node, Exception> {
        self.binary_level(Self::logical_and, |kind| match kind {
            TokenKind::OrOr => Some(BinaryOp::LogicalOr),
            _ => None,
        })
    }

    fn logical_and(&mut self) -> Result<Node, Exception> {
        self.binary_level(Self::equality, |kind| match kind {
            TokenKind::AndAnd => Some(BinaryOp::LogicalAnd),
            _ => None,
        })
    }

    fn binary_level(
        &mut self,
        next: fn(&mut Self) -> Result<Node, Exception>,
        op_for: fn(&TokenKind) -> Option<BinaryOp>,
    ) -> Result<Node, Exception> {
        let (mut left, mut depth) = next(self)?;
        while let Some(op) = op_for(&self.peek().kind) {
            self.pos += 1;
            let (right, right_depth) = next(self)?;
            depth = depth.max(right_depth) + 1;
            if depth > MAX_DEPTH {
                return Err(too_deep());
            }
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Ok((left, depth))
    }

    fn equality(&mut self) -> Result<Node, Exception> {
        self.binary_level(Self::relational, |kind| match kind {
            TokenKind::EqEq => Some(BinaryOp::LooseEq),
            TokenKind::NotEq => Some(BinaryOp::LooseNe),
            TokenKind::EqEqEq => Some(BinaryOp::StrictEq),
            TokenKind::NotEqEq => Some(BinaryOp::StrictNe),
            _ => None,
        })
    }

    fn relational(&mut self) -> Result<Node, Exception> {
        self.binary_level(Self::additive, |kind| match kind {
            TokenKind::Lt => Some(BinaryOp::Lt),
            TokenKind::Gt => Some(BinaryOp::Gt),
            TokenKind::LtEq => Some(BinaryOp::LtEq),
            TokenKind::GtEq => Some(BinaryOp::GtEq),
            _ => None,
        })
    }

    fn additive(&mut self) -> Result<Node, Exception> {
        self.binary_level(Self::multiplicative, |kind| match kind {
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Sub),
            _ => None,
        })
    }

    fn multiplicative(&mut self) -> Result<Node, Exception> {
        self.binary_level(Self::unary, |kind| match kind {
            TokenKind::Star => Some(BinaryOp::Mul),
            TokenKind::Slash => Some(BinaryOp::Div),
            TokenKind::Percent => Some(BinaryOp::Rem),
            _ => None,
        })
    }

    fn nested<T>(
        &mut self,
        inner: impl FnOnce(&mut Self) -> Result<T, Exception>,
    ) -> Result<T, Exception> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(too_deep());
        }
        let result = inner(self);
        self.depth -= 1;
        result
    }

    fn unary(&mut self) -> Result<Node, Exception> {
        let op = match self.peek().kind {
            TokenKind::Plus => UnaryOp::Plus,
            TokenKind::Minus => UnaryOp::Minus,
            TokenKind::Bang => UnaryOp::Not,
            TokenKind::Typeof => UnaryOp::Typeof,
            _ => return self.primary(),
        };
        self.pos += 1;
        let (operand, depth) = self.nested(Self::unary)?;
        if depth + 1 > MAX_DEPTH {
            return Err(too_deep());
        }
        Ok((Expr::Unary(op, Box::new(operand)), depth + 1))
    }

    fn primary(&mut self) -> Result<Node, Exception> {
        let expr = match &self.peek().kind {
            TokenKind::Number(value) => Expr::Number(*value),
            TokenKind::String(value) => Expr::String(value.clone()),
            TokenKind::True => Expr::Boolean(true),
            TokenKind::False => Expr::Boolean(false),
            TokenKind::Null => Expr::Null,
            TokenKind::Identifier(name) => Expr::Identifier(name.clone()),
            TokenKind::LParen => {
                self.pos += 1;
                let node = self.nested(Self::expression)?;
                if self.peek().kind != TokenKind::RParen {
                    return Err(self.unexpected());
                }
                self.pos += 1;
                return Ok(node);
            }
            _ => return Err(self.unexpected()),
        };
        self.pos += 1;
        Ok((expr, 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num(value: f64) -> Box<Expr> {
        Box::new(Expr::Number(value))
    }

    /// Parses a Script of expression statements.
    fn exprs(source: &str) -> Result<Vec<Expr>, Exception> {
        Ok(parse(source)?
            .into_iter()
            .map(|statement| match statement {
                Statement::Expression(expr) => expr,
                other => panic!("not an expression statement: {other:?}"),
            })
            .collect())
    }

    fn syntax_error(source: &str) -> String {
        let error = parse(source).unwrap_err();
        assert_eq!(error.kind, ErrorKind::SyntaxError, "{source}");
        error.message
    }

    #[test]
    fn precedence_and_associativity() {
        use BinaryOp::*;
        // 1 + 2 * 3 - 4 => (1 + (2 * 3)) - 4
        assert_eq!(
            exprs("1 + 2 * 3 - 4").unwrap(),
            vec![Expr::Binary(
                Sub,
                Box::new(Expr::Binary(
                    Add,
                    num(1.0),
                    Box::new(Expr::Binary(Mul, num(2.0), num(3.0)))
                )),
                num(4.0),
            )]
        );
        // 1 < 2 == 3 > 4 => (1 < 2) == (3 > 4)
        assert_eq!(
            exprs("1 < 2 == 3 > 4").unwrap(),
            vec![Expr::Binary(
                LooseEq,
                Box::new(Expr::Binary(Lt, num(1.0), num(2.0))),
                Box::new(Expr::Binary(Gt, num(3.0), num(4.0))),
            )]
        );
        // -2 * (3 + 1) and !-x
        assert_eq!(
            exprs("-2 * (3 + 1); !-x").unwrap(),
            vec![
                Expr::Binary(
                    Mul,
                    Box::new(Expr::Unary(UnaryOp::Minus, num(2.0))),
                    Box::new(Expr::Binary(Add, num(3.0), num(1.0))),
                ),
                Expr::Unary(
                    UnaryOp::Not,
                    Box::new(Expr::Unary(
                        UnaryOp::Minus,
                        Box::new(Expr::Identifier("x".into()))
                    )),
                ),
            ]
        );
        assert_eq!(
            exprs("typeof typeof x").unwrap(),
            vec![Expr::Unary(
                UnaryOp::Typeof,
                Box::new(Expr::Unary(
                    UnaryOp::Typeof,
                    Box::new(Expr::Identifier("x".into()))
                ))
            )]
        );
        assert_eq!(
            exprs("a || b && c == d").unwrap(),
            vec![Expr::Binary(
                BinaryOp::LogicalOr,
                Box::new(Expr::Identifier("a".into())),
                Box::new(Expr::Binary(
                    BinaryOp::LogicalAnd,
                    Box::new(Expr::Identifier("b".into())),
                    Box::new(Expr::Binary(
                        BinaryOp::LooseEq,
                        Box::new(Expr::Identifier("c".into())),
                        Box::new(Expr::Identifier("d".into()))
                    ))
                ))
            )]
        );
    }

    #[test]
    fn statements_and_semicolon_insertion() {
        assert_eq!(exprs("").unwrap(), vec![]);
        assert_eq!(exprs(" ;; ; ").unwrap(), vec![]);
        assert_eq!(exprs("1;2\n3").unwrap().len(), 3);
        // A newline does not end a statement when the next token continues it.
        assert_eq!(
            exprs("1\n+ 2").unwrap(),
            vec![Expr::Binary(BinaryOp::Add, num(1.0), num(2.0))]
        );
    }

    #[test]
    fn declarations_blocks_and_assignment() {
        let binding = |name: &str, initializer: Option<Expr>| VariableBinding {
            name: name.into(),
            initializer,
        };
        assert_eq!(
            parse("var a, b = 1\nlet c = a = b += 2; const d = 1; { c }").unwrap(),
            vec![
                Statement::Declaration(
                    DeclarationKind::Var,
                    vec![binding("a", None), binding("b", Some(Expr::Number(1.0)))]
                ),
                Statement::Declaration(
                    DeclarationKind::Let,
                    vec![binding(
                        "c",
                        Some(Expr::Assign(
                            None,
                            "a".into(),
                            Box::new(Expr::Assign(Some(BinaryOp::Add), "b".into(), num(2.0)))
                        ))
                    )]
                ),
                Statement::Declaration(
                    DeclarationKind::Const,
                    vec![binding("d", Some(Expr::Number(1.0)))]
                ),
                Statement::Block(vec![Statement::Expression(Expr::Identifier("c".into()))]),
            ]
        );
        // `let` not followed by an identifier is an IdentifierReference.
        assert_eq!(
            parse("let; let + 1").unwrap(),
            vec![
                Statement::Expression(Expr::Identifier("let".into())),
                Statement::Expression(Expr::Binary(
                    BinaryOp::Add,
                    Box::new(Expr::Identifier("let".into())),
                    num(1.0)
                )),
            ]
        );
        assert_eq!(
            syntax_error("let x = 1 let y"),
            "Unexpected identifier 'let'"
        );
        assert_eq!(syntax_error("var"), "Unexpected end of input");
        assert_eq!(syntax_error("var 1"), "Unexpected number");
        assert_eq!(syntax_error("const = 1"), "Unexpected token '='");
        assert_eq!(syntax_error("var a,"), "Unexpected end of input");
        assert_eq!(syntax_error("{ var a = }"), "Unexpected token '}'");
    }

    #[test]
    fn if_else_statements() {
        let parsed = parse("if (true) 1; else if (false) 2; else { 3; }").unwrap();
        assert!(matches!(
            parsed.as_slice(),
            [Statement::If {
                condition: Expr::Boolean(true),
                consequent,
                alternate: Some(alternate),
            }] if matches!(&**consequent, Statement::Expression(Expr::Number(1.0)))
                && matches!(&**alternate, Statement::If { .. })
        ));
        assert!(matches!(
            parse("if (true); else;").unwrap().as_slice(),
            [Statement::If {
                consequent,
                alternate: Some(alternate),
                ..
            }] if matches!(&**consequent, Statement::Empty)
                && matches!(&**alternate, Statement::Empty)
        ));
        assert_eq!(
            syntax_error("if (true) let x = 1"),
            "Lexical declaration cannot appear in a single-statement context"
        );
        assert_eq!(
            syntax_error("if (true) 1; else const x = 1"),
            "Lexical declaration cannot appear in a single-statement context"
        );
        assert_eq!(syntax_error("if true 1"), "Unexpected token 'true'");
        assert_eq!(syntax_error("if (true 1"), "Unexpected number");
        assert_eq!(syntax_error("if (true)"), "Unexpected end of input");
        assert_eq!(syntax_error("else 1"), "Unexpected token 'else'");
    }

    #[test]
    fn while_and_loop_control_statements() {
        assert!(matches!(
            parse("while (true) { if (false) continue; break; }")
                .unwrap()
                .as_slice(),
            [Statement::While { body, .. }]
                if matches!(&**body, Statement::Block(statements)
                    if matches!(statements.as_slice(),
                        [Statement::If { .. }, Statement::Break(None)]))
        ));
        assert_eq!(syntax_error("break"), "Illegal break statement");
        assert_eq!(syntax_error("continue"), "Illegal continue statement");
        assert_eq!(syntax_error("while true break"), "Unexpected token 'true'");
        assert_eq!(syntax_error("while (true"), "Unexpected end of input");
        assert_eq!(
            syntax_error("while (true) let x = 1"),
            "Lexical declaration cannot appear in a single-statement context"
        );
        assert_eq!(
            syntax_error("while (true) break label"),
            "Undefined label 'label'"
        );
    }

    #[test]
    fn labelled_break_and_continue() {
        assert!(matches!(
            parse("outer: inner: while (true) { continue outer; break inner; }")
                .unwrap()
                .as_slice(),
            [Statement::Labelled { label, body }]
                if &**label == "outer"
                    && matches!(&**body, Statement::Labelled { label, .. }
                        if &**label == "inner")
        ));
        assert!(parse("done: { break done; }").is_ok());
        assert_eq!(
            syntax_error("missing: { break other; }"),
            "Undefined label 'other'"
        );
        assert_eq!(
            syntax_error("block: { continue block; }"),
            "Illegal continue statement: 'block' does not denote an iteration statement"
        );
        assert_eq!(
            syntax_error("same: { same: while (true) break same; }"),
            "Label 'same' has already been declared"
        );
        assert_eq!(
            syntax_error("label: let x = 1"),
            "Lexical declaration cannot appear in a single-statement context"
        );
        assert_eq!(syntax_error("break missing"), "Undefined label 'missing'");
        assert_eq!(
            syntax_error("continue missing"),
            "Undefined label 'missing'"
        );
    }

    #[test]
    fn throw_try_catch_and_finally_statements() {
        assert!(matches!(
            parse("try { throw 1; } catch (error) { error; } finally { 2; }")
                .unwrap()
                .as_slice(),
            [Statement::Try {
                catch: Some(CatchClause { binding: Some(binding), .. }),
                finally: Some(_),
                ..
            }] if &**binding == "error"
        ));
        assert!(parse("try {} catch {}").is_ok());
        assert!(parse("try {} finally {}").is_ok());
        assert_eq!(syntax_error("throw\n1"), "Illegal newline after throw");
        assert_eq!(
            syntax_error("try {}"),
            "Missing catch or finally after try"
        );
        assert_eq!(syntax_error("try 1 catch {}"), "Unexpected number");
        assert_eq!(
            syntax_error("try {} catch (true) {}"),
            "Unexpected token 'true'"
        );
        assert_eq!(
            syntax_error("try {} catch (e) { let e; }"),
            "Identifier 'e' has already been declared"
        );
    }

    #[test]
    fn malformed_syntax() {
        assert_eq!(syntax_error("1 +"), "Unexpected end of input");
        assert_eq!(syntax_error("1 + * 2"), "Unexpected token '*'");
        assert_eq!(syntax_error("(1 + 2"), "Unexpected end of input");
        assert_eq!(syntax_error("1 + 2)"), "Unexpected token ')'");
        assert_eq!(syntax_error("()"), "Unexpected token ')'");
        assert_eq!(syntax_error("1 2"), "Unexpected number");
        assert_eq!(syntax_error("a b"), "Unexpected identifier 'b'");
        assert_eq!(syntax_error("'a' 'b'"), "Unexpected string");
        assert_eq!(syntax_error("1.2.3"), "Unexpected number");
        assert_eq!(syntax_error("x (1)"), "Call expressions are not supported");
        assert_eq!(syntax_error("1\n(2)"), "Call expressions are not supported");
        assert_eq!(syntax_error("'open"), "Unterminated string literal");
    }

    #[test]
    fn nesting_is_bounded() {
        let ok = format!(
            "{}1{}",
            "(".repeat(MAX_DEPTH - 1),
            ")".repeat(MAX_DEPTH - 1)
        );
        assert!(parse(&ok).is_ok());
        let parens = format!("{}1{}", "(".repeat(100_000), ")".repeat(100_000));
        assert_eq!(parse(&parens).unwrap_err(), too_deep());
        assert_eq!(parse(&"!".repeat(100_000)).unwrap_err(), too_deep());
        let chain = vec!["1"; 100_000].join("+");
        assert_eq!(parse(&chain).unwrap_err(), too_deep());
        let flat = vec!["1"; MAX_DEPTH].join("+");
        assert!(parse(&flat).is_ok());
    }
}
