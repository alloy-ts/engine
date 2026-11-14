# `Context`

A Context is a persistent handle to one Realm. It is not an ECMAScript
execution context. Create it with `engine.createContext()`; `new Context()`
throws.

```ts
type Primitive = undefined | null | boolean | number | string;
```

## Implemented (MVP)

| Member | Behavior |
|---|---|
| `eval(source: string): Primitive` | Parses and evaluates `source` as a Script in the Context's Realm and returns the completion value. A throw completion is thrown as a host `SyntaxError`, `ReferenceError`, `TypeError` or `RangeError`. The Context stays usable after a throw. |
| `get(name: string): Primitive` | Returns a global binding's value; throws `ReferenceError` (`name is not defined`) when it does not exist. |
| `set(name: string, value: Primitive): void` | Creates or updates a global binding. Throws `TypeError` for a value that is not a supported primitive (objects, arrays, functions, symbols, bigints), for a name that is not a usable identifier (`""`, `"a b"`, `"true"`, `"if"`), and for the read-only globals `undefined`, `NaN` and `Infinity`. |
| `dispose(): void` | Releases the Realm and its heap. Idempotent. |
| `disposed: boolean` | Whether `dispose()` has been called. |

After `dispose()`, `eval`, `get` and `set` throw `Error("Context is disposed")`.

Numbers round-trip exactly, including `-0`, `NaN` and `±Infinity`. Strings
are copied in both directions. A host string containing a lone surrogate
(in `set`, `get`, or `eval` source) is rejected with `TypeError`, because
runtime strings are UTF-8.

Error objects are created with the host's `SyntaxError`/`ReferenceError`/
`TypeError`/`RangeError` captured when the addon loads, so later patches of
those globals do not affect them.

## Architectural invariants

- Host values cross the boundary only through checked conversion; no JSON
  bridge.
- Globals are per Context and never mirror the host `globalThis`.

## Planned post-MVP work

- Thrown runtime values other than errors, and object transfer, once the
  runtime has objects.
