# Runtime architecture

Alloy TS embeds its own ECMAScript runtime in a NAPI-RS addon. The host
process (Node.js or Bun) loads the addon; the runtime inside it owns its own
values, Realms and evaluator.

```text
host JS ──► Engine ──► Context (public handle)
                         └─► RuntimeContext (lifecycle)
                               └─► Realm ── global bindings
                                     │   └─ Heap (UTF-8 strings)
                                     └─► ScriptRecord ──► evaluator ──► Completion<Value>
```

## Implemented (MVP)

| Piece | File | Role |
|---|---|---|
| `Engine`, `Context` | `src/engine.rs` | NAPI facade; the only host-boundary conversion |
| `RuntimeContext` | `src/runtime/context.rs` | Owns one Realm; `dispose()` drops it |
| `Realm` | `src/runtime/realm.rs` | Global bindings plus the Heap |
| `Heap` | `src/runtime/heap.rs` | Owns string contents; `StringHandle` indexes it |
| `Value` | `src/runtime/value.rs` | `Undefined`, `Null`, `Boolean`, `Number(f64)`, `String(StringHandle)` |
| `ScriptRecord` | `src/runtime/script.rs` | Parsed Script: a list of expression statements |
| lexer / parser | `src/runtime/lexer.rs`, `parser.rs` | Source text to `ScriptRecord`, early SyntaxErrors |
| evaluator | `src/runtime/evaluator.rs` | `ScriptRecord` to `Completion<Value>` |
| abstract operations | `src/runtime/operations.rs`, `number.rs` | ToNumber, ToString, ToBoolean, IsLessThan, IsLooselyEqual, IsStrictlyEqual, `Number::toString`, StringToNumber |

`Completion<T>` has two variants, `Normal(T)` and `Throw(Exception)`. An
`Exception` records an error kind (`SyntaxError`, `ReferenceError`,
`TypeError`, `RangeError`) and a message, because the runtime has no error
objects yet. `src/engine.rs` turns it into an instance of the host's error
constructor of that kind, captured at addon load.

The supported language subset is listed in [engine-plan.md](engine-plan.md).

## Architectural invariants

- `src/runtime/**` does not depend on napi. Only `src/engine.rs` converts
  between host `napi_value`s and runtime `Value`s.
- The runtime `Value` is separate from the NAPI value wrappers in
  `src/value/**` (`SmlInt`, `Float`, `Integer`, ...) and from host values.
- Values and `StringHandle`s belong to one Realm and never cross Realms or the
  host boundary: strings are copied out as host strings.
- A public `Context` is a persistent Realm-facing handle, not an ECMAScript
  execution context. Each Context owns a separate Realm, so contexts share no
  bindings or heap.
- The runtime never reads or writes the host `globalThis`.
- Host values cross the boundary only through checked conversion: only
  undefined, null, boolean, number and string are accepted; everything else
  is a TypeError. There is no JSON bridge.
- Parser nesting is bounded (`parser::MAX_DEPTH`, 256) so parsing, evaluation
  and dropping the syntax tree cannot overflow the native stack. Exceeding it
  is a RangeError.
- No `unsafe` code in `src/runtime/**` or `src/engine.rs`.

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
