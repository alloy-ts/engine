//! Number conversions that ECMA-262 defines on strings: `Number::toString`
//! (radix 10) and `StringToNumber`. Pure Rust, independent of the host engine.

use num_bigint::BigUint;
use num_traits::ToPrimitive;

/// ECMA-262 WhiteSpace or LineTerminator, as trimmed by `StringToNumber`.
pub fn is_js_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{9}' | '\u{B}' | '\u{C}' | ' ' | '\u{A0}' | '\u{FEFF}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}'
    ) || is_line_terminator(c)
}

/// ECMA-262 LineTerminator.
pub fn is_line_terminator(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

/// `Number::toString(x, 10)` (ECMA-262 §6.1.6.1.20).
///
/// Rust's `{:e}` formatting yields the shortest digit string that round-trips,
/// which is the `s`/`k` the specification asks for; only the layout differs.
pub fn number_to_string(x: f64) -> String {
    if x.is_nan() {
        return "NaN".to_owned();
    }
    if x == 0.0 {
        return "0".to_owned();
    }
    if x < 0.0 {
        return format!("-{}", number_to_string(-x));
    }
    if x.is_infinite() {
        return "Infinity".to_owned();
    }
    let scientific = format!("{x:e}");
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let k = digits.len() as i64;
    let n = exponent.parse::<i64>().unwrap_or(0) + 1;
    if k <= n && n <= 21 {
        format!("{digits}{}", "0".repeat((n - k) as usize))
    } else if 0 < n && n <= 21 {
        let (int, frac) = digits.split_at(n as usize);
        format!("{int}.{frac}")
    } else if -6 < n && n <= 0 {
        format!("0.{}{digits}", "0".repeat((-n) as usize))
    } else {
        let e = n - 1;
        let sign = if e < 0 { '-' } else { '+' };
        let (first, rest) = digits.split_at(1);
        if rest.is_empty() {
            format!("{first}e{sign}{}", e.abs())
        } else {
            format!("{first}.{rest}e{sign}{}", e.abs())
        }
    }
}

/// Exact value of `digits` in `radix`, correctly rounded to f64. Returns
/// `None` when `digits` is empty or contains a non-digit (including `_`).
pub fn parse_radix_digits(digits: &str, radix: u32) -> Option<f64> {
    if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
        return None;
    }
    let value = BigUint::parse_bytes(digits.as_bytes(), radix)?;
    Some(value.to_f64().unwrap_or(f64::INFINITY))
}

/// Whether `s` matches StrUnsignedDecimalLiteral without `Infinity`:
/// `digits [. digits?] | . digits`, then an optional `e[+-]digits`.
pub fn is_unsigned_decimal_literal(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;
    let digits = |i: &mut usize| {
        let start = *i;
        while *i < bytes.len() && bytes[*i].is_ascii_digit() {
            *i += 1;
        }
        *i - start
    };
    let mut mantissa = digits(&mut i);
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        mantissa += digits(&mut i);
    }
    if mantissa == 0 {
        return false;
    }
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
            i += 1;
        }
        if digits(&mut i) == 0 {
            return false;
        }
    }
    i == bytes.len()
}

/// Decimal digits already validated by [`is_unsigned_decimal_literal`].
pub fn parse_decimal(s: &str) -> f64 {
    s.parse::<f64>().unwrap_or(f64::NAN)
}

/// `StringToNumber` (ECMA-262 §7.1.4.1.1).
pub fn string_to_number(s: &str) -> f64 {
    let text = s.trim_matches(is_js_whitespace);
    if text.is_empty() {
        return 0.0;
    }
    if text.len() > 2 && text.as_bytes()[0] == b'0' {
        let radix = match text.as_bytes()[1] {
            b'x' | b'X' => Some(16),
            b'o' | b'O' => Some(8),
            b'b' | b'B' => Some(2),
            _ => None,
        };
        if let Some(radix) = radix {
            return parse_radix_digits(&text[2..], radix).unwrap_or(f64::NAN);
        }
    }
    let (sign, body) = match text.as_bytes()[0] {
        b'+' => (1.0, &text[1..]),
        b'-' => (-1.0, &text[1..]),
        _ => (1.0, text),
    };
    if body == "Infinity" {
        sign * f64::INFINITY
    } else if is_unsigned_decimal_literal(body) {
        sign * parse_decimal(body)
    } else {
        f64::NAN
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_to_string_follows_the_spec_layout() {
        let cases = [
            (0.0, "0"),
            (-0.0, "0"),
            (f64::NAN, "NaN"),
            (f64::INFINITY, "Infinity"),
            (f64::NEG_INFINITY, "-Infinity"),
            (1.0, "1"),
            (-1.5, "-1.5"),
            (0.1 + 0.2, "0.30000000000000004"),
            (123456789.0, "123456789"),
            (1e21, "1e+21"),
            (1e20, "100000000000000000000"),
            (123e18, "123000000000000000000"),
            (1.5e21, "1.5e+21"),
            (0.000001, "0.000001"),
            (1e-7, "1e-7"),
            (1.25e-7, "1.25e-7"),
            (0.00001234, "0.00001234"),
            (f64::MAX, "1.7976931348623157e+308"),
            (5e-324, "5e-324"),
        ];
        for (input, expected) in cases {
            assert_eq!(number_to_string(input), expected, "{input:e}");
        }
    }

    #[test]
    fn string_to_number_follows_string_numeric_literal() {
        let cases = [
            ("", 0.0),
            ("  \n\t ", 0.0),
            ("42", 42.0),
            (" 42 ", 42.0),
            ("\u{FEFF}7\u{2028}", 7.0),
            ("+1.5", 1.5),
            ("-.5", -0.5),
            ("5.", 5.0),
            ("1e3", 1000.0),
            ("1E-2", 0.01),
            ("0x1F", 31.0),
            ("0o17", 15.0),
            ("0b101", 5.0),
            ("Infinity", f64::INFINITY),
            ("-Infinity", f64::NEG_INFINITY),
            ("0x20000000000001", 9007199254740992.0),
        ];
        for (input, expected) in cases {
            assert_eq!(string_to_number(input), expected, "{input:?}");
        }
        assert!(string_to_number("-0").is_sign_negative());
        for nan in [
            "abc", "1a", "1e", "1_000", ".", "+", "-0x10", "0x", "0xG", "infinity", "inf", "NaN",
            "1 2", "\u{85}1",
        ] {
            assert!(string_to_number(nan).is_nan(), "{nan:?}");
        }
    }
}
