# QuickJS `Atom` API Reference

In QuickJS, an **Atom** represents an interned string or symbol used for property lookup or string table keys.

## Overview

Atoms represent property keys in QuickJS. Numerical indices (e.g. `object[1]`) are represented directly as integers, while string or symbol keys (e.g. `object["foo"]` or `object.foo`) are interned atoms in a hash table.

## Structure

```rust
pub struct Atom<'js> {
    pub(crate) atom: qjs::JSAtom,
    pub(crate) ctx: Ctx<'js>,
}
```

## `PredefinedAtom`

QuickJS provides predefined atoms for commonly used JavaScript keywords, property names, and built-in symbols (e.g. `null`, `prototype`, `toString`, `length`, `Symbol.iterator`). Using predefined atoms avoids string hash table lookups.

### Example Predefined Atoms

- `PredefinedAtom::Null` (`"null"`)
- `PredefinedAtom::Prototype` (`"prototype"`)
- `PredefinedAtom::Length` (`"length"`)
- `PredefinedAtom::ToString` (`"toString"`)
- `PredefinedAtom::SymbolIterator` (`"Symbol.iterator"`)

## Key Methods

- `Atom::from_str(ctx, name)`: Creates an atom from a Rust string.
- `Atom::from_u32(ctx, val)`: Creates an atom from a `u32` index.
- `Atom::from_predefined(ctx, predefined)`: Creates an atom from a `PredefinedAtom`.
- `Atom::to_string()`: Converts the atom to a Rust `String`.
- `Atom::to_js_string()`: Converts the atom to a QuickJS `String`.
- `Atom::to_value()`: Converts the atom to a QuickJS `Value`.
