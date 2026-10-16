# QuickJS `Context` API Reference

The `Context` class wraps a single QuickJS execution context (`rquickjs::Context`), holding its own global variables, scope, and evaluation stack.

## Constructors & Factories

### `new Context()`

Creates a full QuickJS context with standard intrinsic objects registered (Object, Array, Function, Promise, Eval, etc.).

---

### `Context.createBase()`

Creates a minimal base QuickJS context with only core required objects.

## Methods

### `eval(code: string): string`

Evaluates a JavaScript code string and returns its string representation.

- **Parameters:**
  - `code` (`string`): The JavaScript code string to execute.
- **Returns:**
  - `string`: Result converted to string (or `"undefined"`/`"null"`).

---

### `evalInt(code: string): number`

Evaluates a JavaScript code string and casts/extracts the result as a 32-bit signed integer (`i32`).

- **Parameters:**
  - `code` (`string`): The JavaScript code to execute.
- **Returns:**
  - `number`: The resulting integer value.

---

### `evalBool(code: string): boolean`

Evaluates a JavaScript code string and extracts the result as a boolean.

- **Parameters:**
  - `code` (`string`): The JavaScript code to execute.
- **Returns:**
  - `boolean`: The resulting boolean value.
