//! Recursive-descent parser for the MVP expression-statement grammar, with
//! ECMA-262 precedence (loosest first): equality, relational, additive,
//! multiplicative, unary, primary.

use super::completion::{ErrorKind, Exception};
use super::lexer::{Token, TokenKind, tokenize};
use super::script::{BinaryOp, Expr, UnaryOp};

/// Maximum syntax-tree depth. Parsing, evaluation and dropping the tree all
/// recurse over it, so the bound keeps hostile input from overflowing the
/// native stack.
pub const MAX_DEPTH: usize = 256;

pub fn parse(source: &str) -> Result<Vec<Expr>, Exception> {
    let tokens = tokenize(source)?;
    let mut parser = Parser {
        tokens,
        pos: 0,
        depth: 0,
    };
    let mut statements = Vec::new();
    loop {
        match parser.peek().kind {
            TokenKind::Eof => return Ok(statements),
            TokenKind::Semicolon => parser.pos += 1,
            _ => {
                statements.push(parser.expression()?.0);
                parser.end_statement()?;
            }
        }
    }
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
        TokenKind::LParen => "(",
        TokenKind::RParen => ")",
        TokenKind::Semicolon => ";",
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
}

impl Parser {
    fn peek(&self) -> &Token {
        // `tokenize` always ends with Eof and `pos` never advances past it.
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn unexpected(&self) -> Exception {
        Exception::syntax(describe(&self.peek().kind))
    }

    /// An ExpressionStatement ends at `;`, at end of input, or before a token
    /// on a new line (automatic semicolon insertion, ECMA-262 §12.10).
    fn end_statement(&mut self) -> Result<(), Exception> {
        let token = self.peek();
        match token.kind {
            TokenKind::Semicolon => {
                self.pos += 1;
                Ok(())
            }
            TokenKind::Eof => Ok(()),
            // Without call expressions, `(` after an operand is never valid,
            // and ASI does not apply to it in ECMAScript either.
            TokenKind::LParen => Err(Exception::syntax("Call expressions are not supported")),
            _ if token.newline_before => Ok(()),
            _ => Err(self.unexpected()),
        }
    }

    fn expression(&mut self) -> Result<Node, Exception> {
        self.equality()
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

    fn nested(
        &mut self,
        inner: fn(&mut Self) -> Result<Node, Exception>,
    ) -> Result<Node, Exception> {
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
            parse("1 + 2 * 3 - 4").unwrap(),
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
            parse("1 < 2 == 3 > 4").unwrap(),
            vec![Expr::Binary(
                LooseEq,
                Box::new(Expr::Binary(Lt, num(1.0), num(2.0))),
                Box::new(Expr::Binary(Gt, num(3.0), num(4.0))),
            )]
        );
        // -2 * (3 + 1) and !-x
        assert_eq!(
            parse("-2 * (3 + 1); !-x").unwrap(),
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
    }

    #[test]
    fn statements_and_semicolon_insertion() {
        assert_eq!(parse("").unwrap(), vec![]);
        assert_eq!(parse(" ;; ; ").unwrap(), vec![]);
        assert_eq!(parse("1;2\n3").unwrap().len(), 3);
        // A newline does not end a statement when the next token continues it.
        assert_eq!(
            parse("1\n+ 2").unwrap(),
            vec![Expr::Binary(BinaryOp::Add, num(1.0), num(2.0))]
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
