# Runtime architecture

`alloy-engine` is Alloy TS's embedded ECMAScript runtime. The runtime in
`src/runtime/` is plain Rust with no napi or Node dependency. The crate's NAPI
facade in `src/engine.rs` exposes it to JavaScript as the `Engine` and
`Context` classes, and is the only place that converts between host values and
runtime [`Value`](api/README.md)s.

```text
RuntimeContext (lifecycle)
  └─ Realm ── Global Environment Record (global object + let/const)
        │   └─ Heap (UTF-8 strings + native error records)
        └─ ScriptRecord ──► evaluator ──► CompletionRecord ──► Completion<Value>
```

## Implemented

| Piece                            | File                                                 | Role                                                                                                           |
| -------------------------------- | ---------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| `RuntimeContext`                 | `src/runtime/context.rs`                             | Owns one Realm; `dispose()` drops it                                                                           |
| `Realm`                          | `src/runtime/realm.rs`                               | `[[GlobalEnv]]` plus the Heap                                                                                  |
| Environment Records              | `src/runtime/environment.rs`                         | Declarative and Global Environment Records                                                                     |
| `Heap`                           | `src/runtime/heap.rs`                                | Owns UTF-8 strings and native error records; Realm-local handles index them                                    |
| `Value`                          | `src/runtime/value.rs`                               | MVP primitives plus `Error(ErrorHandle)`, a temporary object-like value pending the full object model          |
| `ScriptRecord`                   | `src/runtime/script.rs`                              | Parsed Script: statements, declarations, blocks; declared-name static semantics                                |
| lexer / parser                   | `src/runtime/lexer.rs`, `src/runtime/parser.rs`      | Source text to `ScriptRecord`, early SyntaxErrors                                                              |
| evaluator                        | `src/runtime/evaluator.rs`                           | ScriptEvaluation, Global/BlockDeclarationInstantiation, Reference Records, statements to `CompletionRecord`    |
| `CompletionRecord`, `Completion` | `src/runtime/completion.rs`                          | Spec Completion Record and the embedder's view of a script result                                              |
| abstract operations              | `src/runtime/operations.rs`, `src/runtime/number.rs` | ToNumber, ToString, ToBoolean, IsLessThan, IsLooselyEqual, IsStrictlyEqual, `Number::toString`, StringToNumber |

The JavaScript API is documented in [api/engine.md](api/engine.md) and
[api/context.md](api/context.md), the Rust API of the runtime module in
[api/README.md](api/README.md), and the supported language subset in
[engine-plan.md](engine-plan.md).

### Completion Records

`CompletionRecord` follows ECMA-262 §6.2.4 for the completion types the
runtime can produce:

- `[[Type]]`: `normal`, `break`, `continue` or `throw` (`CompletionType`).
  `return` is added with functions.
- `[[Value]]`: normal, break and continue hold a `Value` or _empty_ (`None`).
  A throw completion holds the thrown `Value`. Parser/runtime errors are
  represented by Realm-owned native error handles until ordinary Error
  objects exist.
- `[[Target]]`: empty for normal, throw and unlabeled loop control; labelled
  break/continue carry their label until the matching statement consumes it.

`CompletionRecord::update_empty` is UpdateEmpty (§6.2.4.4). Statement lists
use it, so a script's value is that of its last value-producing statement
(`1; let x = 2;`, `1; {}` and `1; var a;` all produce 1). ScriptEvaluation
turns a remaining empty value into undefined, and embeddings receive the
simpler `Completion<Value>` (`Normal(value)` or `Throw(value)`), which
an embedding (such as the NAPI facade) turns into a host value or host error.

### Environments and bindings

The Realm owns a Global Environment Record (§9.1.1.4) made of:

- an object part over the global object, holding `var` declarations,
  built-ins (`undefined`, `NaN`, `Infinity`; non-writable and
  non-configurable) and properties created by sloppy assignment or by the
  host (`Realm::set_global`; writable and configurable);
- a Declarative Environment Record for top-level `let` / `const`.

Each Block gets a new Declarative Environment Record. Identifier lookup is
ResolveBinding / GetIdentifierReference from the running LexicalEnvironment
outwards, producing a Reference Record that GetValue, PutValue and
InitializeReferencedBinding consume. Uninitialized lexical bindings (the
temporal dead zone) throw ReferenceError; `const` bindings are strict
immutable bindings, so assigning them throws TypeError.

Scripts are evaluated as sloppy code. Assigning an undeclared name creates a
global object property; assigning a read-only global is silently ignored.

GlobalDeclarationInstantiation (§16.1.7) runs before each Script's
statements, against the bindings left by earlier Scripts in the same Realm:
a `let` / `const` name that is already lexically declared or is a
non-configurable global property (a `var` or a built-in), or a `var` name
that is already lexically declared, is a SyntaxError and nothing is
declared. A lexical binding whose Script throws before initializing it stays
in its temporal dead zone for the life of the Realm, as in the spec.

### Deviations from ECMA-262

- There are no objects, so the global object is a map of own data
  properties: no prototype chain (HasProperty is HasOwnProperty), no
  accessors, always extensible (CanDeclareGlobalVar is always true), no
  `[[Enumerable]]`, and not reachable as a value (`globalThis`, `this`). The
  Global Environment Record has no `[[GlobalThisValue]]`.
- There is no execution-context stack. One ScriptEvaluation keeps its block
  environments on a stack whose order is the `[[OuterEnv]]` chain; this is
  enough because nothing (no closures) outlives its block.
- Bindings record no `deletable` flag, because nothing can delete them.
- Native errors are Realm-owned `Value::Error` handles rather than ordinary
  ECMAScript Error objects. They can be caught and inspected with `typeof`,
  but cannot cross the primitive-only host return boundary as normal values.

## Architectural invariants

- `src/runtime/` does not depend on napi, Node, or a JavaScript host. Host
  conversion belongs only in the NAPI facade (`src/engine.rs`).
- The runtime `Value` is separate from host values and from the NAPI value
  wrappers in alloy-ts/core (`SmlInt`, `Float`, `Integer`, ...).
- Values and `StringHandle`s belong to one Realm and never cross Realms or a
  host boundary: embeddings must copy string contents out.
- `RuntimeContext` is a lifecycle wrapper around one Realm, not an ECMAScript
  execution context. Each instance owns a separate Realm, so contexts share no
  bindings or heap.
- The runtime never reads or writes a host `globalThis`.
- Parser nesting is bounded (`parser::MAX_DEPTH`, 256) so parsing, evaluation
  and dropping the syntax tree cannot overflow the native stack. Blocks,
  parentheses, unary operators and assignment chains count towards it.
  Exceeding it is a RangeError.
- No `unsafe` code in the runtime or the NAPI facade.

## Planned work

The ownership boundaries and staged Node compatibility work are defined in
[roadmap.md](roadmap.md).

- Control flow: `for`, `for...in`, and `for...of` loops. Realm-owned native error handles become
  ordinary Error objects when the object model is implemented.
- Objects, functions and prototypes in the runtime, closures and an
  execution-context stack, a real global object, and a garbage-collected
  heap. The Heap only grows until its Realm is dropped.
- CompactCell, literal interning, and the Alloy-owned tracing collector are
  future storage architecture; the current MVP uses a `Value` enum and a
  grow-until-Realm-disposal heap.
- Strings that can hold lone surrogates (UTF-16 or WTF-8 storage).
