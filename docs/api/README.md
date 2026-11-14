# Public Rust API

The crate name is `alloy-engine`; the library is imported as `alloy_engine`.
Modules are private. The public surface is the types and functions re-exported
from the crate root.

```rust
use alloy_engine::{
    Completion, Disposed, ErrorKind, Exception, Heap, Realm, RuntimeContext,
    ScriptRecord, StringHandle, Value, evaluate_script, number_to_string,
};
```

## `RuntimeContext`

Lifecycle wrapper around one [`Realm`](#realm). `new()` / `Default` seeds a
Realm. `dispose()` drops that Realm and its heap and is idempotent.
`is_disposed()` reports that state. `realm()` and `realm_mut()` return
`Err(Disposed)` afterwards; so does `evaluate(source)`, which otherwise
forwards to `Realm::evaluate`.

`Disposed` is a unit error type. `RuntimeContext` is not an ECMAScript
execution context.

## `Realm`

Internal Realm: the global bindings and the heap their strings live in. Each
Realm is independent; a `Value` or `StringHandle` from one Realm must never be
used with another.

`new()` / `Default` seeds the non-writable globals `undefined`, `NaN` and
`Infinity`. `get_global(name)` returns a binding or `None`.
`set_global(name, value)` creates or updates a writable binding and returns a
`TypeError` `Exception` for a name that is not a usable identifier or for a
read-only binding. `alloc_string` interns a string on the Realm's heap and
returns a `Value::String`. `heap()` borrows the heap. `evaluate(source)`
parses a Script and evaluates it, returning `Completion<Value>` (`Throw` for
syntax or runtime errors, without leaving the Realm unusable).

## `Value` and `Heap`

`Value` is the runtime language value, restricted to the MVP primitives:

- `Undefined`
- `Null`
- `Boolean(bool)`
- `Number(f64)`
- `String(StringHandle)`

`StringHandle` indexes a UTF-8 string owned by a Realm's `Heap`. A handle is
only meaningful with the heap that allocated it. `Heap::alloc_string` and
`Heap::string` allocate and borrow those contents. The heap only grows until
its Realm is dropped.

## `Completion`, `Exception`, `ErrorKind`

`Completion<T>` is a minimal stand-in for the ECMA-262 Completion Record: only
`Normal(T)` and `Throw(Exception)` exist, because the MVP has no
`break` / `continue` / `return` or `try` / `finally`.

`Exception` records an `ErrorKind` (`SyntaxError`, `ReferenceError`,
`TypeError`, `RangeError`) and a message. The runtime has no error objects
yet. `ErrorKind::name()` is the constructor name an embedding should throw.

## `ScriptRecord` and `evaluate_script`

`ScriptRecord::parse(source)` produces a parsed Script (a list of expression
statements) or a syntax `Exception`. `evaluate_script(realm, script)` runs an
already-parsed record. Most embeddings should call `Realm::evaluate` or
`RuntimeContext::evaluate`, which parse and evaluate together.

## `number_to_string`

Formats an `f64` with ECMAScript `Number::toString` layout (`"Infinity"`,
`"0"` for `-0`, `"1e-7"`, and so on). Abstract operations such as ToNumber,
ToString, ToBoolean, and the equality comparators are crate-private.

## Language subset

See [../engine-plan.md](../engine-plan.md).
