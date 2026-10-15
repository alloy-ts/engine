# `JsEngine` / `Engine` API Reference

`Engine` (also exported as `JsEngine`) is a JavaScript-visible class that wraps the native N-API context handle (`napi::Engine`). It provides safe methods to interact with the underlying JavaScript runtime context, allocate values, query global scope, inspect N-API host versions, and store environment-scoped instance data.

## Module Import

```ts
import { Engine, JsEngine } from "@alloy-ts/engine";

// `Engine` and `JsEngine` refer to the same binding class.
const engine = new JsEngine();
```

---

## The N-API Context Contract & Lifecycle Safety

`JsEngine` owns and manages an `napi::Engine` handle. It adheres to three strict N-API context contracts:

1. **Context Reuse**: All operations within a `JsEngine` instance execute through the single context handle captured during construction.
2. **Context Isolation**: Each `JsEngine` instance captures its own context. Handle fields are private and cannot be leaked, cached for general reuse, or shared across Worker threads or distinct addon instances.
3. **Automatic Teardown & Disposal Guard**: Upon construction, `JsEngine` registers an environment cleanup hook (`add_engine_cleanup_hook`). When the host environment or addon instance is unloaded, an internal atomic flag marks the engine as disposed. Any subsequent call to an instance method on a disposed engine will safely fail and throw an error rather than accessing dangling pointers.

---

## Constructor

### `new JsEngine()` / `new Engine()`

Constructs a new `JsEngine` instance.

- **Arguments**: None (the N-API runtime automatically injects the active environment context handle).
- **Returns**: A new `JsEngine` instance.

```ts
const engine = new JsEngine();
```

---

## Instance Methods

### `is_disposed(): boolean`

Checks whether the underlying N-API context has been unloaded/disposed.

- **Returns**: `boolean` — `false` while the addon and environment remain loaded; `true` if the addon environment has been torn down.

```ts
if (!engine.is_disposed()) {
  console.log("Engine context is active");
}
```

---

### `create_array(len: number): unknown[]`

Allocates a JavaScript `Array` of the requested length in the engine context.

- **Parameters**:
  - `len` (`number`): The initial length of the array (`u32`).
- **Returns**: `unknown[]` — A new JS Array with `len` items.

```ts
const arr = engine.create_array(5);
console.log(arr.length); // 5
```

---

### `create_string(value: string): string`

Creates a JavaScript string value through the captured N-API context.

- **Parameters**:
  - `value` (`string`): The primitive string value.
- **Returns**: `string` — The resulting JS string.

```ts
const str = engine.create_string("hello world");
console.log(str); // "hello world"
```

---

### `create_int32(int: number): number`

Builds a JavaScript number from a 32-bit signed integer (`i32`).

- **Parameters**:
  - `int` (`number`): Integer value (`i32`).
- **Returns**: `number` — The created JS number.

```ts
const num = engine.create_int32(42);
console.log(num); // 42
```

---

### `create_double(value: number): number`

Builds a JavaScript number from a 64-bit double-precision floating-point number (`f64`).

- **Parameters**:
  - `value` (`number`): Floating-point number (`f64`).
- **Returns**: `number` — The created JS number.

```ts
const doubleVal = engine.create_double(3.14159);
console.log(doubleVal); // 3.14159
```

---

### `has_global(name: string): boolean`

Queries whether a property with the given `name` exists on the global object (`globalThis`).

- **Parameters**:
  - `name` (`string`): The property name to check on the global object.
- **Returns**: `boolean` — `true` if property exists on globalThis; `false` otherwise.

```ts
const hasMath = engine.has_global("Math"); // true
const hasFoo = engine.has_global("nonExistentGlobalProperty"); // false
```

---

### `get_napi_version(): number`

Retrieves the N-API version supported by the host runtime (e.g. Node.js).

- **Returns**: `number` — Host N-API version (u32).

```ts
const version = engine.get_napi_version();
console.log(`Host N-API version: ${version}`);
```

---

### `make_object(): { ok: boolean; engine: string }`

Constructs a JavaScript object using the context and initializes default properties.

- **Returns**: `object` — Object with `{ ok: true, engine: "napi" }`.

```ts
const obj = engine.make_object();
console.log(obj); // { ok: true, engine: 'napi' }
```

---

### `set_instance_data(value: string): void`

Persists VM-specific string state on the N-API Agent (environment instance).

- **Parameters**:
  - `value` (`string`): State string to store on the Agent instance.
- **Returns**: `void`

```ts
engine.set_instance_data("custom-vm-state");
```

---

### `get_instance_data(): string | null`

Reads back the VM-specific state stored on the Agent via `set_instance_data`.

- **Returns**: `string | null` — The stored string, or `null` / `undefined` if no state was set.

```ts
const state = engine.get_instance_data();
console.log(state); // "custom-vm-state"
```
