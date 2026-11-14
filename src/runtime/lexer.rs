//! Tokenizer for the MVP subset of ECMAScript Script source text.

use super::completion::Exception;
use super::number::{
    is_js_whitespace, is_line_terminator, is_unsigned_decimal_literal, parse_decimal,
    parse_radix_digits,
};
use super::script::BinaryOp;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Number(f64),
    String(Box<str>),
    Identifier(Box<str>),
    True,
    False,
    Null,
    Var,
    Const,
    Typeof,
    If,
    Else,
    While,
    Break,
    Continue,
    Throw,
    Try,
    Catch,
    Finally,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    Lt,
    LtEq,
    Gt,
    GtEq,
    EqEq,
    NotEq,
    EqEqEq,
    NotEqEq,
    AndAnd,
    OrOr,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semicolon,
    Colon,
    /// `=`
    Assign,
    /// `+=`, `-=`, `*=`, `/=`, `%=` with the operator they apply.
    CompoundAssign(BinaryOp),
    Eof,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    /// A LineTerminator occurs between this token and the previous one; used
    /// for automatic semicolon insertion.
    pub newline_before: bool,
}

/// ECMA-262 ReservedWords other than `true`, `false` and `null`. `var` and
/// `const` become keyword tokens; every other one is unsupported syntax and
/// rejected rather than treated as a global identifier reference.
const RESERVED_WORDS: &[&str] = &[
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

fn is_identifier_start(c: char) -> bool {
    c == '$' || c == '_' || c.is_alphabetic()
}

fn is_identifier_part(c: char) -> bool {
    is_identifier_start(c) || c.is_alphanumeric() || c == '\u{200C}' || c == '\u{200D}'
}

/// Whether `name` is an identifier the MVP evaluator can reference: an
/// IdentifierName that is neither a reserved word nor a literal keyword.
pub fn is_binding_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(is_identifier_start)
        && chars.all(is_identifier_part)
        && !matches!(name, "true" | "false" | "null")
        && !RESERVED_WORDS.contains(&name)
}

fn unexpected() -> Exception {
    Exception::syntax("Invalid or unexpected token")
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, Exception> {
    Lexer {
        chars: source.chars().collect(),
        pos: 0,
    }
    .run()
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn peek(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn run(mut self) -> Result<Vec<Token>, Exception> {
        let mut tokens = Vec::new();
        loop {
            let newline_before = self.skip_trivia()?;
            let kind = match self.peek(0) {
                None => TokenKind::Eof,
                Some(c) => self.token(c)?,
            };
            let done = kind == TokenKind::Eof;
            tokens.push(Token {
                kind,
                newline_before,
            });
            if done {
                return Ok(tokens);
            }
        }
    }

    /// Skips whitespace and comments, reporting whether a line terminator
    /// was crossed (a multi-line comment containing one counts).
    fn skip_trivia(&mut self) -> Result<bool, Exception> {
        let mut newline = false;
        while let Some(c) = self.peek(0) {
            if is_js_whitespace(c) {
                newline |= is_line_terminator(c);
                self.pos += 1;
            } else if c == '/' && self.peek(1) == Some('/') {
                while self.peek(0).is_some_and(|c| !is_line_terminator(c)) {
                    self.pos += 1;
                }
            } else if c == '/' && self.peek(1) == Some('*') {
                self.pos += 2;
                loop {
                    match self.peek(0) {
                        None => return Err(Exception::syntax("Unterminated comment")),
                        Some('*') if self.peek(1) == Some('/') => {
                            self.pos += 2;
                            break;
                        }
                        Some(c) => {
                            newline |= is_line_terminator(c);
                            self.pos += 1;
                        }
                    }
                }
            } else {
                break;
            }
        }
        Ok(newline)
    }

    fn token(&mut self, c: char) -> Result<TokenKind, Exception> {
        if c.is_ascii_digit() || (c == '.' && self.peek(1).is_some_and(|d| d.is_ascii_digit())) {
            return self.number();
        }
        if c == '"' || c == '\'' {
            return self.string(c);
        }
        if is_identifier_start(c) {
            return self.word();
        }
        let (kind, len) = match (c, self.peek(1), self.peek(2)) {
            ('=', Some('='), Some('=')) => (TokenKind::EqEqEq, 3),
            ('!', Some('='), Some('=')) => (TokenKind::NotEqEq, 3),
            ('&', Some('&'), Some('=')) | ('|', Some('|'), Some('=')) => {
                return Err(Exception::syntax(format!("Unexpected token '{c}{c}='")));
            }
            ('=', Some('='), _) => (TokenKind::EqEq, 2),
            ('!', Some('='), _) => (TokenKind::NotEq, 2),
            ('<', Some('='), _) => (TokenKind::LtEq, 2),
            ('>', Some('='), _) => (TokenKind::GtEq, 2),
            ('&', Some('&'), _) => (TokenKind::AndAnd, 2),
            ('|', Some('|'), _) => (TokenKind::OrOr, 2),
            ('=', Some('>'), _) => return Err(Exception::syntax("Unexpected token '=>'")),
            // Longer punctuators the MVP does not support must not be split
            // into supported ones (`--1` is not `-(-1)` in ECMAScript).
            ('+', Some('+'), _) | ('-', Some('-'), _) | ('*', Some('*'), _) => {
                return Err(Exception::syntax(format!("Unexpected token '{c}{c}'")));
            }
            ('<', Some('<'), _) | ('>', Some('>'), _) => {
                return Err(Exception::syntax(format!("Unexpected token '{c}{c}'")));
            }
            ('+' | '-' | '*' | '/' | '%', Some('='), _) => {
                let operator = match c {
                    '+' => BinaryOp::Add,
                    '-' => BinaryOp::Sub,
                    '*' => BinaryOp::Mul,
                    '/' => BinaryOp::Div,
                    _ => BinaryOp::Rem,
                };
                (TokenKind::CompoundAssign(operator), 2)
            }
            ('=', _, _) => (TokenKind::Assign, 1),
            ('+', _, _) => (TokenKind::Plus, 1),
            ('-', _, _) => (TokenKind::Minus, 1),
            ('*', _, _) => (TokenKind::Star, 1),
            ('/', _, _) => (TokenKind::Slash, 1),
            ('%', _, _) => (TokenKind::Percent, 1),
            ('!', _, _) => (TokenKind::Bang, 1),
            ('<', _, _) => (TokenKind::Lt, 1),
            ('>', _, _) => (TokenKind::Gt, 1),
            ('(', _, _) => (TokenKind::LParen, 1),
            (')', _, _) => (TokenKind::RParen, 1),
            ('{', _, _) => (TokenKind::LBrace, 1),
            ('}', _, _) => (TokenKind::RBrace, 1),
            (',', _, _) => (TokenKind::Comma, 1),
            (';', _, _) => (TokenKind::Semicolon, 1),
            (':', _, _) => (TokenKind::Colon, 1),
            _ => return Err(unexpected()),
        };
        self.pos += len;
        Ok(kind)
    }

    fn take_while(&mut self, predicate: impl Fn(char) -> bool) -> String {
        let start = self.pos;
        while self.peek(0).is_some_and(&predicate) {
            self.pos += 1;
        }
        self.chars[start..self.pos].iter().collect()
    }

    fn number(&mut self) -> Result<TokenKind, Exception> {
        let radix = match (self.peek(0), self.peek(1)) {
            (Some('0'), Some('x' | 'X')) => Some(16),
            (Some('0'), Some('o' | 'O')) => Some(8),
            (Some('0'), Some('b' | 'B')) => Some(2),
            _ => None,
        };
        let value = if let Some(radix) = radix {
            self.pos += 2;
            let digits = self.take_while(|c| c.is_ascii_alphanumeric());
            parse_radix_digits(&digits, radix).ok_or_else(unexpected)?
        } else {
            let start = self.pos;
            self.take_while(|c| c.is_ascii_digit());
            if self.peek(0) == Some('.') {
                self.pos += 1;
                self.take_while(|c| c.is_ascii_digit());
            }
            if matches!(self.peek(0), Some('e' | 'E')) {
                self.pos += 1;
                if matches!(self.peek(0), Some('+' | '-')) {
                    self.pos += 1;
                }
                self.take_while(|c| c.is_ascii_digit());
            }
            let text: String = self.chars[start..self.pos].iter().collect();
            // Legacy octal-like literals (`012`, `08`) are not supported.
            let leading_zero =
                text.len() > 1 && text.starts_with('0') && text.as_bytes()[1].is_ascii_digit();
            if leading_zero || !is_unsigned_decimal_literal(&text) {
                return Err(unexpected());
            }
            parse_decimal(&text)
        };
        // The source character after a NumericLiteral must not be an
        // IdentifierStart or DecimalDigit (`3in`, `1_000`, `0x1g`).
        if self
            .peek(0)
            .is_some_and(|c| is_identifier_start(c) || c.is_ascii_digit() || c == '\\')
        {
            return Err(unexpected());
        }
        Ok(TokenKind::Number(value))
    }

    fn word(&mut self) -> Result<TokenKind, Exception> {
        let word = self.take_while(is_identifier_part);
        if self.peek(0) == Some('\\') {
            // Unicode escapes in identifiers are not supported.
            return Err(unexpected());
        }
        Ok(match word.as_str() {
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            "var" => TokenKind::Var,
            "const" => TokenKind::Const,
            "typeof" => TokenKind::Typeof,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "throw" => TokenKind::Throw,
            "try" => TokenKind::Try,
            "catch" => TokenKind::Catch,
            "finally" => TokenKind::Finally,
            w if RESERVED_WORDS.contains(&w) => {
                return Err(Exception::syntax(format!("Unexpected token '{w}'")));
            }
            _ => TokenKind::Identifier(word.into()),
        })
    }

    fn hex_digits(&mut self, count: usize) -> Result<u32, Exception> {
        let mut value = 0;
        for _ in 0..count {
            let digit = self
                .peek(0)
                .and_then(|c| c.to_digit(16))
                .ok_or_else(invalid_escape)?;
            value = value * 16 + digit;
            self.pos += 1;
        }
        Ok(value)
    }

    /// Code unit or code point after `\u`.
    fn unicode_escape(&mut self) -> Result<u32, Exception> {
        if self.peek(0) != Some('{') {
            return self.hex_digits(4);
        }
        self.pos += 1;
        let digits = self.take_while(|c| c.is_ascii_hexdigit());
        if self.peek(0) != Some('}') || digits.is_empty() {
            return Err(invalid_escape());
        }
        self.pos += 1;
        u32::from_str_radix(&digits, 16)
            .ok()
            .filter(|cp| *cp <= 0x10FFFF)
            .ok_or_else(invalid_escape)
    }

    fn string(&mut self, quote: char) -> Result<TokenKind, Exception> {
        self.pos += 1;
        let mut out = String::new();
        loop {
            let c = match self.peek(0) {
                None | Some('\n' | '\r') => {
                    return Err(Exception::syntax("Unterminated string literal"));
                }
                Some(c) => c,
            };
            self.pos += 1;
            if c == quote {
                return Ok(TokenKind::String(out.into()));
            }
            if c != '\\' {
                out.push(c);
                continue;
            }
            let escaped = self
                .peek(0)
                .ok_or_else(|| Exception::syntax("Unterminated string literal"))?;
            self.pos += 1;
            match escaped {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                'b' => out.push('\u{8}'),
                'f' => out.push('\u{C}'),
                'v' => out.push('\u{B}'),
                '0' if !self.peek(0).is_some_and(|d| d.is_ascii_digit()) => out.push('\0'),
                '0'..='9' => {
                    return Err(Exception::syntax(
                        "Octal escape sequences are not supported",
                    ));
                }
                'x' => {
                    let unit = self.hex_digits(2)?;
                    out.push(char::from_u32(unit).ok_or_else(invalid_escape)?);
                }
                'u' => {
                    let mut cp = self.unicode_escape()?;
                    if (0xD800..0xDC00).contains(&cp)
                        && self.peek(0) == Some('\\')
                        && self.peek(1) == Some('u')
                    {
                        let save = self.pos;
                        self.pos += 2;
                        let low = self.unicode_escape()?;
                        if (0xDC00..0xE000).contains(&low) {
                            cp = 0x10000 + ((cp - 0xD800) << 10) + (low - 0xDC00);
                        } else {
                            self.pos = save;
                        }
                    }
                    // Runtime strings are UTF-8, so lone surrogates cannot
                    // be represented yet.
                    out.push(char::from_u32(cp).ok_or_else(|| {
                        Exception::syntax("Lone surrogate escapes are not supported")
                    })?);
                }
                '\r' => {
                    // LineContinuation; CR LF is a single LineTerminatorSequence.
                    if self.peek(0) == Some('\n') {
                        self.pos += 1;
                    }
                }
                '\n' | '\u{2028}' | '\u{2029}' => {}
                other => out.push(other),
            }
        }
    }
}

fn invalid_escape() -> Exception {
    Exception::syntax("Invalid hexadecimal escape sequence")
}

#[cfg(test)]
mod tests {
    use super::*;
    use TokenKind::*;

    fn kinds(source: &str) -> Vec<TokenKind> {
        tokenize(source)
            .unwrap()
            .into_iter()
            .map(|token| token.kind)
            .collect()
    }

    fn error(source: &str) -> std::string::String {
        let error = tokenize(source).unwrap_err();
        assert_eq!(error.kind, super::super::ErrorKind::SyntaxError);
        error.message
    }

    #[test]
    fn punctuators_and_words() {
        assert_eq!(
            kinds("a: b || c && d === e !== f == g != h <= i >= j < k > l + - * / % ! ( ) ;"),
            vec![
                Identifier("a".into()),
                Colon,
                Identifier("b".into()),
                OrOr,
                Identifier("c".into()),
                AndAnd,
                Identifier("d".into()),
                EqEqEq,
                Identifier("e".into()),
                NotEqEq,
                Identifier("f".into()),
                EqEq,
                Identifier("g".into()),
                NotEq,
                Identifier("h".into()),
                LtEq,
                Identifier("i".into()),
                GtEq,
                Identifier("j".into()),
                Lt,
                Identifier("k".into()),
                Gt,
                Identifier("l".into()),
                Plus,
                Minus,
                Star,
                Slash,
                Percent,
                Bang,
                LParen,
                RParen,
                Semicolon,
                Eof
            ]
        );
        assert_eq!(
            kinds("true false null typeof if else while break continue $_x1 ünï"),
            vec![
                True,
                False,
                Null,
                Typeof,
                If,
                Else,
                While,
                Break,
                Continue,
                Identifier("$_x1".into()),
                Identifier("ünï".into()),
                Eof
            ]
        );
    }

    #[test]
    fn declaration_and_assignment_tokens() {
        assert_eq!(
            kinds("var const let { } , = += -= *= /= %= == ==="),
            vec![
                Var,
                Const,
                Identifier("let".into()),
                LBrace,
                RBrace,
                Comma,
                Assign,
                CompoundAssign(BinaryOp::Add),
                CompoundAssign(BinaryOp::Sub),
                CompoundAssign(BinaryOp::Mul),
                CompoundAssign(BinaryOp::Div),
                CompoundAssign(BinaryOp::Rem),
                EqEq,
                EqEqEq,
                Eof
            ]
        );
        // Unsupported assignment operators are not split into supported ones.
        assert_eq!(error("a **= 1"), "Unexpected token '**'");
        assert_eq!(error("a <<= 1"), "Unexpected token '<<'");
        assert_eq!(error("a &= 1"), "Invalid or unexpected token");
    }

    #[test]
    fn numeric_literals() {
        assert_eq!(
            kinds("0 42 3.25 .5 5. 1e3 2E-2 1e+2 0x1F 0XfF 0o17 0b101"),
            vec![
                Number(0.0),
                Number(42.0),
                Number(3.25),
                Number(0.5),
                Number(5.0),
                Number(1000.0),
                Number(0.02),
                Number(100.0),
                Number(31.0),
                Number(255.0),
                Number(15.0),
                Number(5.0),
                Eof
            ]
        );
        assert_eq!(kinds("1e400"), vec![Number(f64::INFINITY), Eof]);
        for bad in [
            "1e", "1e+", "3in", "1_000", "0x", "0xG", "0b2", "012", "08", "1a",
        ] {
            assert_eq!(error(bad), "Invalid or unexpected token", "{bad}");
        }
    }

    #[test]
    fn string_literals_and_escapes() {
        assert_eq!(
            kinds(
                r#"'a"b' "c'd" "\n\t\r\b\f\v\0\\\"\'" '\x41\u0042\u{43}\u{1F600}\uD83D\uDE00' '\q'"#
            ),
            vec![
                String("a\"b".into()),
                String("c'd".into()),
                String("\n\t\r\u{8}\u{C}\u{B}\0\\\"'".into()),
                String("ABC😀😀".into()),
                String("q".into()),
                Eof
            ]
        );
        assert_eq!(
            kinds("'a\\\nb' 'c\\\r\nd'"),
            vec![String("ab".into()), String("cd".into()), Eof]
        );
        assert_eq!(kinds("'\u{2028}'"), vec![String("\u{2028}".into()), Eof]);
        assert_eq!(error("'abc"), "Unterminated string literal");
        assert_eq!(error("\"abc\ndef\""), "Unterminated string literal");
        assert_eq!(error("'abc\\"), "Unterminated string literal");
        assert_eq!(error(r"'\x4'"), "Invalid hexadecimal escape sequence");
        assert_eq!(
            error(r"'\u{110000}'"),
            "Invalid hexadecimal escape sequence"
        );
        assert_eq!(
            error(r"'\uD800'"),
            "Lone surrogate escapes are not supported"
        );
        assert_eq!(error(r"'\01'"), "Octal escape sequences are not supported");
    }

    #[test]
    fn trivia_and_line_terminators() {
        let tokens = tokenize("1 // c\n2 /* a\nb */ 3 /* x */ 4").unwrap();
        let newlines: Vec<bool> = tokens.iter().map(|t| t.newline_before).collect();
        assert_eq!(newlines, vec![false, true, true, false, false]);
        assert_eq!(error("/* open"), "Unterminated comment");
    }

    #[test]
    fn unsupported_syntax_is_rejected() {
        assert_eq!(error("a => 1"), "Unexpected token '=>'");
        assert_eq!(error("a ||= 1"), "Unexpected token '||='");
        assert_eq!(error("a &&= 1"), "Unexpected token '&&='");
        assert_eq!(error("--1"), "Unexpected token '--'");
        assert_eq!(error("a++"), "Unexpected token '++'");
        assert_eq!(error("2 ** 3"), "Unexpected token '**'");
        assert_eq!(error("[1]"), "Invalid or unexpected token");
        assert_eq!(error("`t`"), "Invalid or unexpected token");
        assert_eq!(error("a\\u0062"), "Invalid or unexpected token");
    }

    #[test]
    fn binding_identifiers() {
        for good in ["x", "$", "_a1", "ünï", "undefined", "NaN", "let"] {
            assert!(is_binding_identifier(good), "{good}");
        }
        for bad in ["", "1x", "a b", "true", "null", "if", "typeof", "a-b"] {
            assert!(!is_binding_identifier(bad), "{bad}");
        }
    }
}
