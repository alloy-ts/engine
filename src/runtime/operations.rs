//! ECMA-262 abstract operations for the MVP values. Native error handles use
//! their Error-style string form until the full object/ToPrimitive model.

use std::borrow::Cow;
use std::cmp::Ordering;

use super::heap::Heap;
use super::number::{number_to_string, string_to_number};
use super::value::Value;

/// ToNumber (§7.1.4).
pub fn to_number(heap: &Heap, value: Value) -> f64 {
    match value {
        Value::Undefined => f64::NAN,
        Value::Null => 0.0,
        Value::Boolean(b) => f64::from(u8::from(b)),
        Value::Number(n) => n,
        Value::String(s) => string_to_number(heap.string(s)),
        Value::Error(error) => string_to_number(&format_error(heap, error)),
    }
}

/// ToBoolean (§7.1.2).
pub fn to_boolean(heap: &Heap, value: Value) -> bool {
    match value {
        Value::Undefined | Value::Null => false,
        Value::Boolean(b) => b,
        Value::Number(n) => !(n == 0.0 || n.is_nan()),
        Value::String(s) => !heap.string(s).is_empty(),
        Value::Error(_) => true,
    }
}

/// ToString (§7.1.17).
pub fn to_string(heap: &Heap, value: Value) -> Cow<'_, str> {
    match value {
        Value::Undefined => "undefined".into(),
        Value::Null => "null".into(),
        Value::Boolean(true) => "true".into(),
        Value::Boolean(false) => "false".into(),
        Value::Number(n) => number_to_string(n).into(),
        Value::String(s) => heap.string(s).into(),
        Value::Error(error) => format_error(heap, error).into(),
    }
}

/// IsStrictlyEqual (§7.2.15).
pub fn is_strictly_equal(heap: &Heap, x: Value, y: Value) -> bool {
    match (x, y) {
        (Value::Undefined, Value::Undefined) | (Value::Null, Value::Null) => true,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        // Number::equal: NaN is unequal to itself and +0 equals -0.
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::String(a), Value::String(b)) => heap.string(a) == heap.string(b),
        (Value::Error(a), Value::Error(b)) => a == b,
        _ => false,
    }
}

fn format_error(heap: &Heap, handle: super::value::ErrorHandle) -> String {
    let error = heap.error(handle);
    format!("{}: {}", error.kind.name(), error.message)
}

/// IsLooselyEqual (§7.2.14) for primitives.
pub fn is_loosely_equal(heap: &Heap, x: Value, y: Value) -> bool {
    match (x, y) {
        (Value::Undefined | Value::Null, Value::Undefined | Value::Null) => true,
        (Value::Undefined | Value::Null, _) | (_, Value::Undefined | Value::Null) => false,
        (Value::Number(a), Value::String(_)) => a == to_number(heap, y),
        (Value::String(_), Value::Number(b)) => to_number(heap, x) == b,
        (Value::Boolean(_), _) => is_loosely_equal(heap, Value::Number(to_number(heap, x)), y),
        (_, Value::Boolean(_)) => is_loosely_equal(heap, x, Value::Number(to_number(heap, y))),
        _ => is_strictly_equal(heap, x, y),
    }
}

/// IsLessThan (§7.2.13) with primitive operands, `LeftFirst` irrelevant
/// because conversions have no side effects. `None` is the spec's
/// `undefined` result (a NaN operand).
pub fn is_less_than(heap: &Heap, x: Value, y: Value) -> Option<bool> {
    if let (Value::String(a), Value::String(b)) = (x, y) {
        // Strings compare by UTF-16 code units, not by code points or bytes.
        let order = heap
            .string(a)
            .encode_utf16()
            .cmp(heap.string(b).encode_utf16());
        return Some(order == Ordering::Less);
    }
    let (a, b) = (to_number(heap, x), to_number(heap, y));
    if a.is_nan() || b.is_nan() {
        None
    } else {
        Some(a < b)
    }
}
