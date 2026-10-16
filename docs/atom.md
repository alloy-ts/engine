# Atom Documentation

`Atom` provides NAPI bindings to QuickJS atoms (`rquickjs::Atom`).

## Overview

In QuickJS, atoms represent interned strings or property index keys (numbers or string identifiers).

## API Reference

### Static Factories

#### `Atom.fromString(ctx: Context, name: string): Atom`

Creates an `Atom` from a string in the given `Context`.

```ts
import { Atom, Context } from "@alloy-ts/engine";

const ctx = new Context();
const atom = Atom.fromString(ctx, "foo");
console.log(atom.toString()); // "foo"
```

#### `Atom.fromU32(ctx: Context, val: number): Atom`

Creates an `Atom` from an unsigned integer index key in the given `Context`.

```ts
const atom = Atom.fromU32(ctx, 42);
console.log(atom.toString()); // "42"
```

### Methods

#### `to_string(): string`

Returns the string representation of the `Atom`.
