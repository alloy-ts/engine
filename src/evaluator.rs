//! Tree-walking evaluator producing `Completion<Value>`.

use super::completion::{Completion, ErrorKind, Exception};
use super::operations::{
    is_less_than, is_loosely_equal, is_strictly_equal, to_boolean, to_number, to_string,
};
use super::realm::Realm;
use super::script::{BinaryOp, Expr, ScriptRecord, UnaryOp};
use super::value::Value;

/// ScriptEvaluation, simplified: the completion value is the value of the
/// last evaluated expression statement, or undefined for an empty script.
pub fn evaluate_script(realm: &mut Realm, script: &ScriptRecord) -> Completion<Value> {
    let mut completion = Value::Undefined;
    for statement in script.statements() {
        match evaluate(realm, statement) {
            Ok(value) => completion = value,
            Err(error) => return Completion::Throw(error),
        }
    }
    Completion::Normal(completion)
}

fn evaluate(realm: &mut Realm, expr: &Expr) -> Result<Value, Exception> {
    Ok(match expr {
        Expr::Number(n) => Value::Number(*n),
        Expr::String(s) => realm.alloc_string(s.clone()),
        Expr::Boolean(b) => Value::Boolean(*b),
        Expr::Null => Value::Null,
        Expr::Identifier(name) => realm.get_global(name).ok_or_else(|| {
            Exception::new(ErrorKind::ReferenceError, format!("{name} is not defined"))
        })?,
        Expr::Unary(op, operand) => {
            let value = evaluate(realm, operand)?;
            let heap = realm.heap();
            match op {
                UnaryOp::Plus => Value::Number(to_number(heap, value)),
                UnaryOp::Minus => Value::Number(-to_number(heap, value)),
                UnaryOp::Not => Value::Boolean(!to_boolean(heap, value)),
            }
        }
        Expr::Binary(op, left, right) => {
            let left = evaluate(realm, left)?;
            let right = evaluate(realm, right)?;
            binary(realm, *op, left, right)
        }
    })
}

fn binary(realm: &mut Realm, op: BinaryOp, x: Value, y: Value) -> Value {
    let heap = realm.heap();
    let number = |f: fn(f64, f64) -> f64| Value::Number(f(to_number(heap, x), to_number(heap, y)));
    match op {
        BinaryOp::Add => {
            if matches!(x, Value::String(_)) || matches!(y, Value::String(_)) {
                let joined = format!("{}{}", to_string(heap, x), to_string(heap, y));
                return realm.alloc_string(joined);
            }
            number(|a, b| a + b)
        }
        BinaryOp::Sub => number(|a, b| a - b),
        BinaryOp::Mul => number(|a, b| a * b),
        BinaryOp::Div => number(|a, b| a / b),
        // Rust's `%` on f64 is the truncating remainder Number::remainder
        // specifies (the sign follows the dividend).
        BinaryOp::Rem => number(|a, b| a % b),
        BinaryOp::Lt => Value::Boolean(is_less_than(heap, x, y) == Some(true)),
        BinaryOp::Gt => Value::Boolean(is_less_than(heap, y, x) == Some(true)),
        BinaryOp::LtEq => Value::Boolean(is_less_than(heap, y, x) == Some(false)),
        BinaryOp::GtEq => Value::Boolean(is_less_than(heap, x, y) == Some(false)),
        BinaryOp::LooseEq => Value::Boolean(is_loosely_equal(heap, x, y)),
        BinaryOp::LooseNe => Value::Boolean(!is_loosely_equal(heap, x, y)),
        BinaryOp::StrictEq => Value::Boolean(is_strictly_equal(heap, x, y)),
        BinaryOp::StrictNe => Value::Boolean(!is_strictly_equal(heap, x, y)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Evaluates `source` in `realm` and renders the result as JavaScript
    /// would display it, prefixing the type so `"1"` and `1` differ.
    fn run_in(realm: &mut Realm, source: &str) -> String {
        match realm.evaluate(source) {
            Completion::Normal(Value::String(s)) => format!("string:{}", realm.heap().string(s)),
            Completion::Normal(Value::Number(n)) if n == 0.0 && n.is_sign_negative() => {
                "number:-0".to_owned()
            }
            Completion::Normal(value) => {
                let kind = match value {
                    Value::Undefined => "undefined",
                    Value::Null => "null",
                    Value::Boolean(_) => "boolean",
                    Value::Number(_) => "number",
                    Value::String(_) => unreachable!(),
                };
                format!("{kind}:{}", to_string(realm.heap(), value))
            }
            Completion::Throw(error) => format!("{}: {}", error.kind.name(), error.message),
        }
    }

    fn run(source: &str) -> String {
        run_in(&mut Realm::new(), source)
    }

    fn check(cases: &[(&str, &str)]) {
        for (source, expected) in cases {
            assert_eq!(run(source), *expected, "{source}");
        }
    }

    #[test]
    fn empty_and_multi_statement_scripts() {
        check(&[
            ("", "undefined:undefined"),
            ("  \n // comment\n", "undefined:undefined"),
            (";;;", "undefined:undefined"),
            ("1; 2; 3", "number:3"),
            ("1\n2\n'three'", "string:three"),
            ("1;;", "number:1"),
            ("1\n+2", "number:3"),
        ]);
    }

    #[test]
    fn precedence_and_unary() {
        check(&[
            ("1 + 2 * 3", "number:7"),
            ("(1 + 2) * 3", "number:9"),
            ("10 - 4 - 3", "number:3"),
            ("2 * 3 % 4", "number:2"),
            ("12 / 2 / 3", "number:2"),
            ("-2 * -3", "number:6"),
            ("- -1", "number:1"),
            ("+'42'", "number:42"),
            ("+''", "number:0"),
            ("+' 0x10 '", "number:16"),
            ("+'abc'", "number:NaN"),
            ("+true", "number:1"),
            ("+null", "number:0"),
            ("+undefined", "number:NaN"),
            ("-null", "number:-0"),
            ("!0", "boolean:true"),
            ("!''", "boolean:true"),
            ("!'0'", "boolean:false"),
            ("!NaN", "boolean:true"),
            ("!!-0", "boolean:false"),
            ("!null", "boolean:true"),
            ("!!undefined", "boolean:false"),
            ("1 + 2 < 4 === true", "boolean:true"),
            ("!1 == 0", "boolean:true"),
        ]);
    }

    #[test]
    fn string_concatenation_and_coercion() {
        check(&[
            ("'a' + 'b'", "string:ab"),
            ("1 + '2'", "string:12"),
            ("'1' + 2", "string:12"),
            ("1 + 2 + '3'", "string:33"),
            ("'1' + 2 + 3", "string:123"),
            ("'' + null + undefined + true", "string:nullundefinedtrue"),
            ("'' + 0.1 * 3", "string:0.30000000000000004"),
            ("'' + -0", "string:0"),
            ("'' + 1e21", "string:1e+21"),
            ("'' + 1 / 3", "string:0.3333333333333333"),
            ("'x' + -Infinity", "string:x-Infinity"),
            ("'' + NaN", "string:NaN"),
            ("'6' * '7'", "number:42"),
            ("'10' - 1", "number:9"),
            ("'a' - 1", "number:NaN"),
            ("true + 1", "number:2"),
            ("null + 1", "number:1"),
            ("undefined + 1", "number:NaN"),
            ("'é' + '😀'", "string:é😀"),
        ]);
    }

    #[test]
    fn number_edge_cases() {
        check(&[
            ("1 / 0", "number:Infinity"),
            ("-1 / 0", "number:-Infinity"),
            ("1 / -0", "number:-Infinity"),
            ("0 / 0", "number:NaN"),
            ("Infinity - Infinity", "number:NaN"),
            ("Infinity * 0", "number:NaN"),
            ("-0", "number:-0"),
            ("0 * -1", "number:-0"),
            ("-0 + 0", "number:0"),
            ("-0 - 0", "number:-0"),
            ("5 % 3", "number:2"),
            ("-5 % 3", "number:-2"),
            ("5 % -3", "number:2"),
            ("-4 % 2", "number:-0"),
            ("5 % 0", "number:NaN"),
            ("5 % Infinity", "number:5"),
            ("Infinity % 2", "number:NaN"),
            ("5.5 % 2", "number:1.5"),
            ("0.1 + 0.2", "number:0.30000000000000004"),
            ("1e308 * 10", "number:Infinity"),
            ("NaN", "number:NaN"),
            ("Infinity", "number:Infinity"),
            ("undefined", "undefined:undefined"),
            ("null", "null:null"),
        ]);
    }

    #[test]
    fn comparisons() {
        check(&[
            ("1 < 2", "boolean:true"),
            ("2 <= 2", "boolean:true"),
            ("3 > 2", "boolean:true"),
            ("2 >= 3", "boolean:false"),
            ("NaN < 1", "boolean:false"),
            ("NaN >= 1", "boolean:false"),
            ("NaN <= NaN", "boolean:false"),
            ("-0 < 0", "boolean:false"),
            ("-0 <= 0", "boolean:true"),
            ("-Infinity < Infinity", "boolean:true"),
            ("'a' < 'b'", "boolean:true"),
            ("'B' < 'a'", "boolean:true"),
            ("'10' < '9'", "boolean:true"),
            ("'10' < 9", "boolean:false"),
            ("'abc' < 'abcd'", "boolean:true"),
            ("'' < 'a'", "boolean:true"),
            ("'a' <= 'a'", "boolean:true"),
            ("'a' >= 'b'", "boolean:false"),
            // U+FF61 sorts after U+1F600 by UTF-16 code units (0xFF61 > 0xD83D)
            // although it is the smaller code point.
            ("'\\uFF61' > '😀'", "boolean:true"),
            ("null < 1", "boolean:true"),
            ("null >= 0", "boolean:true"),
            ("undefined < 1", "boolean:false"),
            ("undefined >= 0", "boolean:false"),
            ("true > false", "boolean:true"),
            ("'x' < 1", "boolean:false"),
            ("'x' >= 1", "boolean:false"),
        ]);
    }

    #[test]
    fn strict_and_loose_equality() {
        check(&[
            ("1 === 1", "boolean:true"),
            ("1 === '1'", "boolean:false"),
            ("1 == '1'", "boolean:true"),
            ("'a' === 'a'", "boolean:true"),
            ("'a' == 'b'", "boolean:false"),
            ("0 == ''", "boolean:true"),
            ("'' == '0'", "boolean:false"),
            ("0 == '0'", "boolean:true"),
            ("' \\n' == 0", "boolean:true"),
            ("'0x10' == 16", "boolean:true"),
            ("null == undefined", "boolean:true"),
            ("null === undefined", "boolean:false"),
            ("null == 0", "boolean:false"),
            ("undefined == 0", "boolean:false"),
            ("null == false", "boolean:false"),
            ("undefined == ''", "boolean:false"),
            ("null == null", "boolean:true"),
            ("undefined === undefined", "boolean:true"),
            ("true == 1", "boolean:true"),
            ("true === 1", "boolean:false"),
            ("false == '0'", "boolean:true"),
            ("false == ''", "boolean:true"),
            ("true == '1'", "boolean:true"),
            ("true == 'true'", "boolean:false"),
            ("true == true", "boolean:true"),
            ("NaN == NaN", "boolean:false"),
            ("NaN === NaN", "boolean:false"),
            ("NaN != NaN", "boolean:true"),
            ("NaN !== NaN", "boolean:true"),
            ("0 === -0", "boolean:true"),
            ("0 == -0", "boolean:true"),
            ("1 !== 1", "boolean:false"),
            ("1 != '2'", "boolean:true"),
            ("'NaN' == NaN", "boolean:false"),
        ]);
    }

    #[test]
    fn errors_are_throw_completions() {
        check(&[
            ("missing", "ReferenceError: missing is not defined"),
            ("1; missing; 2", "ReferenceError: missing is not defined"),
            ("a + b", "ReferenceError: a is not defined"),
            ("globalThis", "ReferenceError: globalThis is not defined"),
            ("1 +", "SyntaxError: Unexpected end of input"),
            ("1 + 2)", "SyntaxError: Unexpected token ')'"),
            ("(1 + 2", "SyntaxError: Unexpected end of input"),
            ("'abc", "SyntaxError: Unterminated string literal"),
            ("1; 2 +", "SyntaxError: Unexpected end of input"),
            ("x = 1", "SyntaxError: Invalid or unexpected token"),
        ]);
    }

    #[test]
    fn syntax_errors_are_reported_before_evaluation() {
        let mut realm = Realm::new();
        // A later syntax error means no statement runs (early errors).
        assert_eq!(
            run_in(&mut realm, "missing; 1 +"),
            "SyntaxError: Unexpected end of input"
        );
    }

    #[test]
    fn identifiers_resolve_from_realm_globals() {
        let mut realm = Realm::new();
        realm.set_global("x", Value::Number(20.0)).unwrap();
        let name = realm.alloc_string("Ada");
        realm.set_global("name", name).unwrap();
        assert_eq!(run_in(&mut realm, "x * 2 + 2"), "number:42");
        assert_eq!(run_in(&mut realm, "'hi ' + name"), "string:hi Ada");
        assert_eq!(run("x"), "ReferenceError: x is not defined");
    }
}
