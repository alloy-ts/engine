# `JsEngine` API Reference

`JsEngine` represents a JavaScript engine instance wrapping an underlying N-API context handle.

## Overview

```rust
use crate::{sys, JsGlobal, Result};
use super::Array;

pub use crate::JsEngine;
```

## Methods

### `create_array(&self, len: u32) -> Result<Array<'_>>`

Allocates a new JavaScript array of the specified length `len` attached to the engine's N-API environment.

- **Parameters:**
  - `len` (`u32`): The initial length/capacity of the array.
- **Returns:**
  - `Result<Array<'_>>`: The created `Array` object wrapped in a `Result`.

---

### `get_global(&self) -> Result<JsGlobal<'static>>`

Retrieves the global JavaScript object from the engine's N-API environment.

- **Returns:**
  - `Result<JsGlobal<'static>>`: The global JavaScript object wrapped in a `Result`.
