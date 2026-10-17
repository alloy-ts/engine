# Context API Documentation and Parity Review

## Overview
The `Context` class represents an active ECMAScript execution realm and state.

## Static Constructors
- `Context.full()`: Creates a context with all standard ECMAScript intrinsic objects and built-ins.
- `Context.base()`: Creates a minimal context with base intrinsics.
- `Context.custom(intrinsics)`: Creates a context with customized intrinsic subsets.
- `Context.builder()`: Returns a context builder instance for fine-grained configuration.
- `Context.fromRaw(ptr, engine)`: Reconstructs a context from a raw context pointer (`BigInt`) and parent Engine handle.

## Prototype Methods
- `Context.prototype.asRaw()`: Returns the raw QuickJS context pointer as a `BigInt`.
- `Context.prototype.getEnginePtr()`: Returns the raw parent engine pointer as a `BigInt`.
- `Context.prototype.Engine()`: Returns the parent `Engine` instance associated with this context.
- `Context.prototype.with(callback)`: Executes a callback passing an inner `JsCtx` handle to perform batch operations efficiently under a single engine lock.
- `Context.prototype.eval(source)`: Evaluates JavaScript source code string and returns the JSON-serialized result.
- `Context.prototype.eval_module(source)`: Evaluates ES module code and returns its namespace object.
- `Context.prototype.json_parse(json)`: Parses JSON string inside the engine.
- `Context.prototype.global_keys()`: Returns array of property keys on the global object.
- `Context.prototype.run_gc()`: Triggers garbage collection cycle.
- `Context.prototype.execute_pending_job()`: Executes one pending promise job / microtask.
- `Context.prototype.has_exception()`: Returns boolean indicating whether a JavaScript exception is pending.
- `Context.prototype.catch()`: Catches and returns any pending JavaScript exception.

## Evaluation Parity
Calling `ctx.eval("console.log(3 + 3.33333)")` or `ctx.eval("3 + 3.33333")` evaluates code in strict mode within the context and marshals primitives and JSON-safe objects back to Node.js.
