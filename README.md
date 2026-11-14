# alloy-ts core

Rust and TypeScript runtime experiments for Alloy TS, built with
[NAPI-RS](https://napi.rs/) and Node-API. The crate explores prototype-linked
value objects and compact in-memory representations while preserving the
observable behavior of JavaScript's built-in types.

## Data model

The runtime separates a value's JavaScript meaning from its in-memory layout.
For example, a JavaScript string is always semantically a string, even when a
runtime represents a short literal with a literal-pool ID or a dynamic string
with a contiguous UTF-8/UTF-16 block. A string literal is never a JavaScript
`Symbol`; the two types can share compact handle machinery while retaining
distinct tags and behavior. See
[`docs/data-types.md`](docs/data-types.md) for the current model and boundaries.

The Rust value layer is in [`src/value`](src/value). It includes target-aware
cells, integer and floating-point representations, Boolean values, prototype
helpers, and the UTF-8/UTF-16 string block draft. String handles are owned by a
`StringTable`; do not retain them after that table is dropped.

The addon does not redefine JavaScript's `typeof`, replace built-in globals, or
patch host prototypes. Addon constructors and helpers are separate APIs, and
bare JavaScript primitives retain their standard ECMAScript behavior.

## Development

Install JavaScript dependencies and build the NAPI addon:

```sh
npm ci
npm run build
```

Run the repository checks with:

```sh
npm run check
npm test
```

The NAPI toolchain and Rust target must be installed for native builds. Runtime
support depends on the selected Node-API backend and target; the Rust in-memory
cell model does not imply zero-copy conversion to a host JavaScript engine's
string storage.
