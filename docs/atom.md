# Atom API

The `Atom` class represents a QuickJS Atom (interned string or index representation used for property keys and symbols).

## Overview

```ts
import { Atom } from "@alloy-ts/engine";

const atom = new Atom("foo");
console.log(atom.to_string()); // "foo"
```

## Methods

- **`new Atom(value: string)`** / **`Atom.from_str(value: string): Atom`**
  Creates an atom from a string.

- **`Atom.from_u32(val: number): Atom`**
  Creates an atom from a `u32` index.

- **`Atom.from_i32(val: number): Atom`**
  Creates an atom from an `i32` index.

- **`Atom.from_bool(val: boolean): Atom`**
  Creates an atom from a boolean value.

- **`Atom.from_f64(val: number): Atom`**
  Creates an atom from an `f64` number.

- **`to_string(): string`**
  Converts the atom to its string representation.

- **`to_value(): string`**
  Converts the atom to its string value.
