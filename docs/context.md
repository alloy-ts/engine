# Context

The `Context` class (also exported as `JSContext`) represents a single QuickJS execution context with its own global variables and stack.

## Construction

```ts
import { Context, Engine } from "@alloy-ts/engine";

// Create context associated with an existing Engine instance
const engine = new Engine();
const ctx = Context.full(engine);

// Or create a base context with minimal intrinsics
const baseCtx = Context.base(engine);

// Or create a self-contained default context
const defaultCtx = new Context();
```

## Methods

- **`eval(source: string, options?: EvalOptions): unknown`**: Evaluates JavaScript code inside global context and returns the result value.
- **`eval_promise(source: string): unknown`**: Evaluates script supporting top-level `await` and drives pending jobs until completion before returning the resolved value.
- **`compile(source: string, filename: string, strict?: boolean): void`**: Compiles a script without executing it, throwing on syntax errors.
- **`json_parse(json: string): unknown`**: Parses a JSON string into a JavaScript value representation within the context.
- **`json_stringify(value: unknown): string | null`**: Stringifies a JavaScript value to JSON.
- **`execute_pending_job(): boolean`**: Executes the next pending job in the runtime queue.
- **`run_gc(): void`**: Triggers cyclic garbage collection on the underlying runtime.
