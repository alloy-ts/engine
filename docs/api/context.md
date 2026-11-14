# `Context`

A Context is a persistent handle to one Realm. It is not an ECMAScript
execution context. Create it with `engine.createContext()`; `new Context()`
throws.

```ts
type Primitive = undefined | null | boolean | number | string;
```

## Implemented

| Member                                      | Behavior                                                                                                                                                                                                                                                                                                                    |
| ------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `eval(source: string): Primitive`           | Parses and evaluates `source` as a Script in the Context's Realm and returns the completion value. Declarations persist across calls. Runtime error handles are thrown as matching host error objects; an explicit primitive `throw` preserves that primitive at the host boundary. The Context stays usable after a throw. |
| `get(name: string): Primitive`              | Returns what a global identifier reference to `name` would: a global `let`/`const` binding, else a global object property. Throws `ReferenceError` when the name is not defined (`name is not defined`) or is in its temporal dead zone (`Cannot access 'name' before initialization`).                                     |
| `set(name: string, value: Primitive): void` | Creates or updates a global binding. Throws `TypeError` for a value that is not a supported primitive (objects, arrays, functions, symbols, bigints), for a name that is not a usable identifier (`""`, `"a b"`, `"true"`, `"if"`), for the read-only globals `undefined`, `NaN` and `Infinity`, and for a `const` binding. |
| `dispose(): void`                           | Releases the Realm and its heap. Idempotent.                                                                                                                                                                                                                                                                                |
| `disposed: boolean`                         | Whether `dispose()` has been called.                                                                                                                                                                                                                                                                                        |

After `dispose()`, `eval`, `get` and `set` throw `Error("Context is disposed")`.

Numbers round-trip exactly, including `-0`, `NaN` and `±Infinity`. Strings
are copied in both directions. A host string containing a lone surrogate
(in `set`, `get`, or `eval` source) is rejected with `TypeError`, because
runtime strings are UTF-8.

Error objects are created with the host's `SyntaxError`/`ReferenceError`/
`TypeError`/`RangeError` captured when the addon loads, so later patches of
those globals do not affect them.

## Architectural invariants

- Host values cross the boundary only through checked conversion in
  `src/engine.rs`; no JSON bridge.
- Globals are per Context and never mirror the host `globalThis`.

## Planned work

- Thrown runtime values other than errors, and object transfer, once the
  runtime has objects.
