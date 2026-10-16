---
title: 'Atom'
description: QuickJS interned strings, symbols, and property keys representation.
---

# Atom

An `Atom` in QuickJS represents an interned string, symbol, or property key.

## Usage

```ts
import { Atom } from "@alloy-ts/engine";

// Create from string
const atom1 = new Atom("foo");
const atom2 = Atom.from_str("bar");

// Create from numbers or booleans
const atom3 = Atom.from_u32(42);
const atom4 = Atom.from_i32(-10);
const atom5 = Atom.from_f64(3.14);
const atom6 = Atom.from_bool(true);

// Convert back to string
console.log(atom1.to_string()); // "foo"
```
