# Public Rust API

The crate name is `alloy-engine`; the library is imported as `alloy_engine`.
The runtime lives in the `alloy_engine::runtime` module. Its submodules are
private; the public surface is the types and functions re-exported from
`alloy_engine::runtime`.

```rust
use alloy_engine::runtime::{
    Completion, CompletionRecord, CompletionType, Disposed, ErrorKind,
    Exception, Heap, Realm, RuntimeContext, ScriptRecord, StringHandle, Value,
    evaluate_script, number_to_string,
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

Internal Realm: its Global Environment Record and the heap its strings live
in. Each Realm is independent; a `Value` or `StringHandle` from one Realm
must never be used with another. Declarations persist across evaluations in
the same Realm.

- `new()` / `Default` seeds the non-writable, non-configurable globals
  `undefined`, `NaN` and `Infinity`.
- `get_binding_value(name)` returns what a global identifier reference
  would: the global `let` / `const` binding if one exists, else the global
  object property (`var`s, built-ins, host-set and implicit globals). It
  returns a `ReferenceError` `Exception` for an unknown name
  (`x is not defined`) or a binding in its temporal dead zone
  (`Cannot access 'x' before initialization`).
- `get_global(name)` is `get_binding_value(name).ok()`: `None` in both of
  those cases.
- `set_global(name, value)` assigns the binding a global reference to
  `name` would resolve to, as strict code would, except that a missing name
  is created as a writable, configurable global object property. Errors: a
  `TypeError` for a name that is not a usable identifier, a read-only global
  (`undefined`, `NaN`, `Infinity`) or a `const`; a `ReferenceError` for a
  `let` / `const` in its temporal dead zone. A later Script may shadow a
  host-created global with `let` / `const` (it is configurable); after that,
  `set_global` writes the lexical binding.
- `alloc_string` copies a string onto the Realm's heap and returns a
  `Value::String`. `heap()` borrows the heap.
- `evaluate(source)` parses a Script and evaluates it, returning
  `Completion<Value>` (`Throw` for syntax or runtime errors, without leaving
  the Realm unusable).

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

## `Completion`, `CompletionRecord`, `CompletionType`, `Exception`, `ErrorKind`

`Completion<T>` is what embeddings receive from evaluating a Script:
`Normal(T)` with the script's completion value, or `Throw(Exception)`. Its
variants are unchanged from the MVP.

`CompletionRecord` is the ECMA-262 Completion Record the evaluator works
with: `Normal(Option<Value>)` (`None` is _empty_) or `Throw(Exception)`.
`completion_type()` returns `[[Type]]` (`CompletionType::Normal` or
`Throw`), `target()` returns `[[Target]]` (always `None` for these types),
`is_abrupt()` reports a non-normal type, `update_empty(value)` is
UpdateEmpty, and `CompletionRecord::EMPTY` is NormalCompletion(empty).
`Completion::<Value>::from(record)` applies ScriptEvaluation's final step,
turning an empty value into undefined. `break`, `continue` and `return`
types arrive with the statements that produce them.

`Exception` records an `ErrorKind` (`SyntaxError`, `ReferenceError`,
`TypeError`, `RangeError`) and a message. The runtime has no error objects
yet. `ErrorKind::name()` is the constructor name an embedding should throw.

## `ScriptRecord` and `evaluate_script`

`ScriptRecord::parse(source)` produces a parsed Script or a syntax
`Exception`, including the early errors that need only that Script
(duplicate lexical declarations, invalid assignment targets).
`evaluate_script(realm, script)` runs an already-parsed record. It runs
GlobalDeclarationInstantiation first, so a conflict with declarations from
earlier Scripts in the Realm is a `SyntaxError` thrown before any statement
runs. Most embeddings should call `Realm::evaluate` or
`RuntimeContext::evaluate`, which parse and evaluate together.

## `number_to_string`

Formats an `f64` with ECMAScript `Number::toString` layout (`"Infinity"`,
`"0"` for `-0`, `"1e-7"`, and so on). Abstract operations such as ToNumber,
ToString, ToBoolean, and the equality comparators are crate-private.

## Language subset

See [../engine-plan.md](../engine-plan.md).
