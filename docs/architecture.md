# Runtime architecture

`alloy-engine` is Alloy TS's embedded ECMAScript runtime. It is a pure Rust
library crate. A host process that wants to expose the runtime to JavaScript
loads the NAPI addon in alloy-ts/core; that addon is the only place that
converts between host values and runtime [`Value`](api/README.md)s.

```text
RuntimeContext (lifecycle)
  └─ Realm ── global bindings
        │   └─ Heap (UTF-8 strings)
        └─ ScriptRecord ──► evaluator ──► Completion<Value>
```

## Implemented (MVP)

| Piece | File | Role |
|---|---|---|
| `RuntimeContext` | `src/context.rs` | Owns one Realm; `dispose()` drops it |
| `Realm` | `src/realm.rs` | Global bindings plus the Heap |
| `Heap` | `src/heap.rs` | Owns string contents; `StringHandle` indexes it |
| `Value` | `src/value.rs` | `Undefined`, `Null`, `Boolean`, `Number(f64)`, `String(StringHandle)` |
| `ScriptRecord` | `src/script.rs` | Parsed Script: a list of expression statements |
| lexer / parser | `src/lexer.rs`, `src/parser.rs` | Source text to `ScriptRecord`, early SyntaxErrors |
| evaluator | `src/evaluator.rs` | `ScriptRecord` to `Completion<Value>` |
| abstract operations | `src/operations.rs`, `src/number.rs` | ToNumber, ToString, ToBoolean, IsLessThan, IsLooselyEqual, IsStrictlyEqual, `Number::toString`, StringToNumber |

The public crate API is listed in [api/README.md](api/README.md).

`Completion<T>` has two variants, `Normal(T)` and `Throw(Exception)`. An
`Exception` records an error kind (`SyntaxError`, `ReferenceError`,
`TypeError`, `RangeError`) and a message, because the runtime has no error
objects yet. A host embedding (such as alloy-ts/core) turns that into a host
error of the matching kind.

The supported language subset is listed in [engine-plan.md](engine-plan.md).

## Architectural invariants

- This crate does not depend on napi, Node, or a JavaScript host. Host
  conversion belongs in the embedding (alloy-ts/core's `src/engine.rs`).
- The runtime `Value` is separate from host values and from the NAPI value
  wrappers in alloy-ts/core (`SmlInt`, `Float`, `Integer`, ...).
- Values and `StringHandle`s belong to one Realm and never cross Realms or a
  host boundary: embeddings must copy string contents out.
- `RuntimeContext` is a lifecycle wrapper around one Realm, not an ECMAScript
  execution context. Each instance owns a separate Realm, so contexts share no
  bindings or heap.
- The runtime never reads or writes a host `globalThis`.
- Parser nesting is bounded (`parser::MAX_DEPTH`, 256) so parsing, evaluation
  and dropping the syntax tree cannot overflow the native stack. Exceeding it
  is a RangeError.
- No `unsafe` code in this crate.

## Planned post-MVP work

- Replace `Completion` with the specification Completion Record
  (`[[Type]]`, `[[Value]]`, `[[Target]]`) when `break`, `continue`, `return`
  or `try`/`finally` arrive, including UpdateEmpty and finally-block
  completion replacement as ECMA-262 specifies. Thrown values then become
  runtime Values (error objects) instead of kind plus message.
- Lexical environments and an execution-context stack, with declarations and
  assignment.
- Objects, functions and prototypes in the runtime, and a garbage-collected
  heap. The MVP Heap only grows until its Realm is dropped.
- Strings that can hold lone surrogates (UTF-16 or WTF-8 storage).
