# Context API Documentation

The `Context` class represents an isolated execution realm in the embedded QuickJS JavaScript engine.

## Creation

### Standalone Context
```ts
import { Context } from "@alloy-ts/engine";
const ctx = new Context();
```
Spins up its own private embedded QuickJS engine.

### Shared Engine Context
```ts
import { Engine } from "@alloy-ts/engine";
const engine = new Engine();
const ctx = engine.create_context(); // or engine.createContext()
```
Creates a context sharing the `Engine` runtime.

---

## Methods

### Code Evaluation
- `ctx.eval(code: string)`: Evaluates a JS script string and returns the result marshalled to a JSON-compatible JS value.
- `ctx.eval_module(code: string)`: Evaluates an ES module string (`import`/`export`) and returns the module namespace object.

### Reflection & Helpers
- `ctx.global_keys()`: Returns an array of keys on the realm's `globalThis`.
- `ctx.json_parse(jsonString: string)`: Parses JSON string inside the QuickJS engine.

### Exception Handling
- `ctx.has_exception()`: Returns `boolean` indicating if a JS exception is pending in this context.
- `ctx.catch()`: Catches and returns the pending JS exception marshalled as a JS value.

### Runtime Operations
- `ctx.run_gc()`: Triggers garbage collection on this context's engine.
- `ctx.execute_pending_job()`: Executes a single pending microtask in this context.
