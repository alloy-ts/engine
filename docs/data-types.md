# Alloy TS data types

This document describes the value model being drafted in the Rust/NAPI-RS
runtime. It distinguishes **semantic type**—what JavaScript and JSON-facing APIs
observe—from **physical representation**—how an Alloy TS runtime may keep a
value in memory. A compact cell is an implementation detail; it does not create
new ECMAScript primitive types.

## Semantic types and storage

| Value family | JavaScript meaning | Draft runtime representation |
|---|---|---|
| `undefined` | `undefined` | Immediate singleton tag |
| `null` | `object` under JavaScript `typeof`; JSON Schema `null` | Immediate singleton tag |
| `Bool` | Boolean wrapper over a boolean primitive | Immediate boolean payload; optional branded wrapper at the NAPI boundary |
| `SmlInt`, `SmlNum`, `Float` | Number wrappers; unboxed results are JavaScript Numbers | Small target-width payloads for Sml values; f64 for `Float` |
| `Integer` | Addon wrapper over a JavaScript BigInt primitive | Inline signed integer when it fits the cell, otherwise a contiguous limb block |
| `BigNum` | Addon object for extended-precision numeric formats | Contiguous block with format metadata and exact encoded payload |
| `String` | JavaScript string primitive; `typeof` is `"string"` | Literal-pool ID for a selected static literal, or immutable UTF-8/UTF-16 block |
| `Symbol` | JavaScript symbol primitive; `typeof` is `"symbol"` | Not yet represented in `CellKind`; any future handle must be distinct from string literals |
| `Object`, arrays | JavaScript objects | Object/array data and property storage owned by the runtime or host engine |
| `Buffer` | Node Buffer, a Uint8Array subtype | Buffer/typed-array backing storage; copying may be required at runtime boundaries |

`SmlInt` and `SmlNum` are constrained-precision members of the Number family;
they do not define new JavaScript primitive types. `Integer` is separately
BigInt-backed and preserves BigInt arithmetic rules. In particular, mixed
BigInt/Number arithmetic continues to throw as specified by ECMAScript.

## Target-width cells

The current memory module uses an eight-byte `ScalarCell` on 64-bit targets and
a four-byte `CompactCell` on 32-bit targets. The 32-bit form stores a compact
tag and payload and uses arena handles for values that need more space. Neither
form stores truncated virtual addresses: cells and handles are valid only
within the arena or table that owns them.

On 64-bit targets, `SmlInt` carries a 32-bit value and `SmlNum` rounds through
binary32. On 32-bit targets, these small-value policies narrow to 16 bits and
binary16. The JavaScript primitive exposed by a numeric wrapper remains a
Number. `Float` retains f64 precision, and `Integer` can spill to arbitrary-size
BigInt limbs rather than silently overflow its inline representation.

## Strings: literal pool and contiguous blocks

The host JavaScript engine may intern literals and property names, but Node-API
does not expose a portable handle to that internal pool. An ahead-of-time
compiler can place referenced string literals in a read-only literal pool and
remove unreferenced entries before runtime initialization. Such a literal can
be represented by a compact `StringLiteralId`; it does not need a per-value
runtime allocation. This is a string storage optimization, not conversion to a
JavaScript `Symbol`.

String literals and JavaScript Symbols have different semantics and must keep
different type tags even if they share an ID allocator or handle table:

| Property | String literal | JavaScript Symbol |
|---|---|---|
| `typeof` | `"string"` | `"symbol"` |
| Equality | By string contents | By unique symbol identity |
| Key behavior | String property key | Symbol property key, distinct from every string key |
| Runtime representation | Read-only literal-pool reference or string block | Separate identity handle |

The current `StringTable::literal_utf8` and `literal_utf16` methods are a
runtime prototype: they deduplicate input in a table and return
`StringLiteralId`s. They do not yet consume compiler-emitted literal IDs or
avoid all initialization. An AOT compiler integration can replace that
registration path with direct references into its read-only pool. The current
literal-deduplication threshold is an implementation detail, not a language guarantee.
JavaScript Symbols are also not yet implemented in the value-cell model.

Materialized strings use `StringBlock`: one immutable contiguous allocation
with a 16-byte metadata prefix followed by the encoded text bytes.

| Encoding | Payload | Notes |
|---|---|---|
| ASCII | One byte per code unit | The bytes are also valid UTF-8. |
| UTF-8 | UTF-8 bytes | Can be borrowed as Rust `&str` without transcoding. |
| UTF-16LE | Little-endian 16-bit code units | Preserves unpaired surrogates and matches JavaScript string indexing. |

The header records byte length, UTF-16 code-unit length, encoding, and a cached
hash over UTF-16 code units. That lets UTF-8 and UTF-16 blocks compare by the
same JavaScript string contents. UTF-16 code-unit length is the counterpart of
JavaScript's `.length`; it is not the count of Unicode scalar values or UTF-8
bytes. When two compile-time literals have the same contents, the compiler may
deduplicate their pool storage; that still preserves string value equality and
does not give them Symbol identity.

`StringTable::concat` and `StringTable::slice` produce contiguous blocks. Slice
indices are interpreted in UTF-16 code units, so slicing a surrogate pair can
produce a block containing an unpaired surrogate. `to_utf8_lossy()` replaces
such malformed UTF-16 with U+FFFD; use the original UTF-16 units when exact
JavaScript string contents must be preserved.

`StringBlock` provides storage and conversion primitives, not a complete
JavaScript String implementation. Search, regular expressions, normalization,
case conversion, rope strings, and zero-copy slices are not promised by this
draft. Crossing the NAPI boundary may require copying or transcoding because
the host engine owns its string representation.

## NAPI boundary and built-ins

The Rust data model does not alter JavaScript's `typeof`, equality operators,
or built-in constructors. In particular, addon code must not replace
`globalThis.String`, `String.prototype`, `Number`, or `BigInt`. JavaScript
string literals and values received through NAPI are host-engine strings;
turning them into `StringBlock`s is an explicit conversion owned by the addon.

An eventual addon string API should keep the built-in `String` callable and
preserve ordinary primitive behavior. Wrapper-specific methods should be
additive and documented separately from methods on bare JavaScript strings.
