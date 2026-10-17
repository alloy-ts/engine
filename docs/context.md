# Context

The `Context` class represents a single execution context (realm) with its own global variables and stack in QuickJS.

## Constructor

```ts
import { Context } from "@alloy-ts/engine";

const ctx = new Context();
```

---

## Methods

### `eval(source: string): unknown`

Evaluates `source` as a global script in strict mode and returns the result as a JSON-compatible value.

```ts
const sum = ctx.eval("10 + 20"); // 30
```

---

### `eval_module(source: string): unknown`

Evaluates `source` as an ES module (`import`/`export` allowed) and returns its module namespace object.

```ts
const ns = ctx.eval_module("export const x = 42;");
console.log(ns.x); // 42
```

---

### `json_parse(json: string): unknown`

Parses `json` inside the embedded engine and returns the resulting value.

```ts
const obj = ctx.json_parse('{"name": "Alloy", "version": 1}');
```

---

### `global_keys(): string[]`

Returns an array of enumerable own property names on the global object (`globalThis`).

```ts
ctx.eval("this.customGlobal = true;");
const keys = ctx.global_keys();
```
