# Atom API

The `Atom` class represents a QuickJS Atom (interned string or index representation used for property keys and symbols).

## Overview

```ts
import { Atom } from "@alloy-ts/engine";

const atom = Atom.fromU32(123);
console.log(atom.toString()); // "123"
```

## Static Factories

- **`new Atom(value: string)`**
  Constructs an atom from a string.

- **`Atom.from_value(val: any): Atom` / `Atom.fromValue(val: any): Atom`**
  Creates an atom from a JS value (primitive or object).

- **`Atom.from_u32(val: number): Atom` / `Atom.fromU32(val: number): Atom`**
  Creates an atom from a `u32` index.

- **`Atom.from_i32(val: number): Atom` / `Atom.fromI32(val: number): Atom`**
  Creates an atom from an `i32` index.

- **`Atom.from_bool(val: boolean): Atom` / `Atom.fromBool(val: boolean): Atom`**
  Creates an atom from a boolean value.

- **`Atom.from_f64(val: number): Atom` / `Atom.fromF64(val: number): Atom`**
  Creates an atom from an `f64` number.

- **`Atom.from_str(val: string): Atom` / `Atom.fromStr(val: string): Atom`**
  Creates an atom from a string.

- **`Atom.from_predefined(name: string): Atom` / `Atom.fromPredefined(name: string): Atom`**
  Creates an atom from a predefined atom name.

## Instance Methods

- **`to_string(): string` / `toString(): string`**
  Converts the atom to its string representation.

- **`to_value(): string` / `toValue(): string`**
  Converts the atom to its string value.
