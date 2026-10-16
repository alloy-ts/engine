# Context API Documentation

The `Context` class provides an execution environment for QuickJS scripts using an associated `Engine`.

## Static Factories

- `Context.full(engine: Engine): Context`: Creates a context initialized with all standard intrinsics.
- `Context.base(engine: Engine): Context`: Creates a minimal base context.

## Methods

- `eval(source: string): string`: Evaluates the JavaScript source string within the context and returns the stringified result.
