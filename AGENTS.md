# Prototype-chain bindings with napi-rs v3

This crate exposes prototype-linked JavaScript objects backed by private Rust
state. Runtime behavior and generated TypeScript declarations must describe the
same public API.

## Authoritative references

Check these references before adding FFI bindings or declaration overrides:

- <https://napi.rs/docs/concepts/types-overwrite>
- <https://napi.rs/docs/concepts/object>
- <https://napi.rs/docs/concepts/type-conversions>
- <https://napi.rs/docs/concepts/values>
- <https://napi.rs/docs/concepts/function>
- <https://napi.rs/docs/concepts/exports>
- <https://napi.rs/docs/concepts/env>
- <https://napi.rs/docs/concepts/reference>
- <https://napi.rs/docs/concepts/understanding-lifetime>
- <https://napi.rs/docs/concepts/module-init>
- <https://napi.rs/docs/concepts/typed-array>
- <https://nodejs.org/api/n-api.html>

The pinned `napi`, `napi-derive`, `napi-build`, and CLI versions are the final
authority for APIs not shown by those pages. Inspect crate source or docs.rs and
run `cargo check`; use `napi::sys` only when the high-level crate lacks the
required operation.

## Runtime object model

- Do not use `#[napi] struct` or `#[napi(constructor)]` for domain types. Build
  callable constructors with `create_function_from_closure` so their instances
  can remain ordinary objects carrying private `napi_wrap` slots.
- Do not use `#[napi(object)]` for domain types. It is a cloned, by-value data
  shape, not a prototype-linked object.
- Each domain type consists of a callable constructor and one prototype. The
  numeric branches are intentionally separate from the host Number and BigInt
  branches:
  - `SmlInt.prototype -> Number.prototype -> Object.prototype`;
  - `SmlNum.prototype -> Number.prototype -> Object.prototype`;
  - `Float.prototype -> Number.prototype -> Object.prototype`;
  - `Integer.prototype -> BigInt.prototype -> Object.prototype`;
  - `BigFlt.prototype -> Object.prototype`;
  - `Bool.prototype` and `Char.prototype` -> `Object.prototype`.
- `new Type(...)`, `Type.new(...)`, and `Type.of(...)` create branded wrappers.
  `Type.cast(...)` returns a bare primitive without wrapper allocation.
- `Value` is the addon-wide brand/contract namespace. It is not a shared
  prototype ancestor: `Value[Symbol.hasInstance]` checks the registered native
  brands across all branches.
- A constructor's `prototype` property is non-writable, non-enumerable, and
  non-configurable.
- Constructors inherit from the host `Function.prototype`. Do not replace or
  patch host `Number`, `BigInt`, `Boolean`, `String`, or their prototypes.
- Shared behavior is composed through flat descriptor mixins, not a synthetic
  common prototype ancestor.
- Set every `[[Prototype]]` once. Do not put a `Proxy` in a prototype chain.
- Traits are flat descriptor mixins copied at initialization with
  `Reflect.ownKeys`, `Reflect.getOwnPropertyDescriptor`, and
  `Reflect.defineProperty`; JavaScript has single inheritance.

## Private state

- The `SmlInt` and `SmlNum` backing slots are `Copy`, no larger than 32 bytes,
  and attached to genuine boxed Number objects with `napi_wrap`.
- `IntegerSlot` is attached to a genuine boxed BigInt object. `BigFltSlot`
  is attached to an ordinary object because ECMAScript has no BigFloat object
  type.
- Spawn boxed Number values with `coerce_to_object()` on a JavaScript number;
  spawn boxed BigInt values with `Object(bigint)` before setting the custom
  prototype and wrapping. Verify that `napi_wrap` succeeds for these boxed
  objects in Node/V8, Bun, and JavaScriptCore before relying on the behavior.
- Only the shared `spawn` helper may create and wrap a scalar instance.
- Wrap once and before freezing. Never wrap a prototype or namespace object.
- Read a slot by value. Never expose `&mut T` from an unwrap performed through a
  shared environment.
- A failed unwrap is the brand check and must become a JavaScript `TypeError`.
- Dynamic slots containing JavaScript values use a captured `WeakMap`.
- Do not store native state in visible own properties.
- `Reflect.ownKeys(SmlInt.of(42))` must return an empty array.

## Buffer values

- Public byte values are named `Buffer`, not `Bytes`.
- Do not replace the host's callable `globalThis.Buffer` with a namespace
  object. Export the package API as `Buffer`; use a distinct global name only if
  a global installation is required.
- Create an actual Buffer-backed value with `BufferSlice::from_data`. Merely
  placing an ordinary object under `Buffer.prototype` does not give it Buffer's
  native brand.
- A custom Buffer prototype may inherit from the host `Buffer.prototype` only
  when instances remain genuine Buffer/typed-array values.
- External backing storage may fall back to copying on runtimes that cannot
  expose it safely; do not promise universal zero-copy behavior.

## Universal scalar contract

Every numeric/value prototype exposes:

1. `valueOf()`
2. `toString(...)`
3. `equals(other)`
4. `toBuffer(bigEndian?)`
5. `[Symbol.toPrimitive](hint)`, returning a primitive only; `ToNumeric` then
   performs ECMAScript Number/BigInt conversion.
6. `[Symbol.toStringTag]`

`BigFlt.prototype[Symbol.toPrimitive]` throws `TypeError` for `"number"` and
`"default"` hints to prevent silent precision loss, and returns a decimal
string for the `"string"` hint.

`SmlInt` and `SmlNum` are reduced-width members of the Number family. Their
payloads are 32-bit on 64-bit targets and 16-bit on 32-bit targets. The
JavaScript wrappers still carry `[[NumberData]]` and expose Number primitives.
`Float` keeps full f64 precision. `SmlFlt` is an alias of `SmlNum`; neither
small name changes the logical numeric family. `Integer`/BigInt values use the
normal target cell width inline when possible and contiguous heap chunks when
they exceed it.

Methods are writable, non-enumerable, and configurable. Accessors are
non-enumerable and configurable. `Symbol.toStringTag` is non-writable,
non-enumerable, and configurable.

## Lifetimes and per-environment state

- Bind returned JavaScript handles explicitly to `'env`.
- Do not transmute handle lifetimes. If a callback cannot return a scoped
  handle, restructure it or return an owned Rust value.
- Use a handle scope inside loops that create many temporary JavaScript values.
- Capture host intrinsics once in `#[napi(module_exports)]` before user code can
  patch them.
- Store `Host` with `Env::set_instance_data`; never use a process-global static
  or thread-local for JavaScript handles.
- Although `get_instance_data` yields a mutable static reference, treat `Host`
  as immutable after initialization except during controlled setup/teardown.
- Create persistent references with `Object::create_ref` or
  `Function::create_ref` and release every owned reference with `unref` in the
  instance-data finalizer or environment cleanup hook. Dropping a reference is
  not cleanup.
- `module_init` is process/library initialization and has no JavaScript
  environment. All prototypes, globals, references, and worker-specific state
  belong in `module_exports`.

## Strict scalar inputs

- `SmlInt.of`: SmlInt, bigint, or finite integral Number only, then enforce
  the target small integer width (i32 on 64-bit, i16 on 32-bit). Parsing is
  exclusive to its parser.
- `SmlNum.of` / `SmlFlt.of`: SmlNum, Number, or BigInt only; values are rounded
  to f32 on 64-bit targets and binary16 on 32-bit targets.
- `Integer.of`: Integer, BigInt, or safe integral Number only.
- `BigFlt.of`: BigFlt, string, Number, or BigInt according to its explicit
  parser contract; implicit numeric coercion must not lose precision.
- `Bool.of`: Bool or boolean only.
- `Char.of`: Char, one-scalar string, or valid Unicode code point only.
- Reject strings, booleans, null, undefined, and plain objects where they are
  not explicitly listed.
- `Option<T>` accepts both null and undefined. Use explicit union wrappers when
  that distinction matters.
- Check the `lossless` result of bigint getters. Do not assume arbitrary
  JavaScript bigint input can be represented by Rust integer arguments.

## TypeScript declaration generation

The repository contains no hand-written `.ts` or `.d.ts` source used solely for
native declarations. `napi build --dts index.d.ts` generates the distributable
declaration file.

### Supported attribute overrides

Use declaration overrides only when the public TypeScript contract intentionally
differs from the Rust conversion type:

- `ts_args_type` replaces the complete generated parameter list.
- `ts_arg_type` replaces one parameter type while preserving inference for the
  others.
- `ts_return_type` replaces the generated return type.
- `ts_type` replaces a field type on a generated object shape.

These attributes affect generated TypeScript only. They do not change runtime
conversion, validation, exports, or prototype behavior. Keep every override
aligned with runtime tests.

### Do not create declaration-only runtime classes

Do not declare placeholder `#[napi] struct` classes with `unreachable!()`
methods and then overwrite their exports in `module_exports`. A `#[napi]` class
is a real runtime class registration, can expose callable code accidentally,
produces class/constructor semantics that do not match this crate's plain
namespace objects, and creates duplicate/conflicting export metadata.

Attach `ts_args_type` and `ts_return_type` to the real exported Rust functions
where possible. For the prototype-only namespace surface that napi-rs cannot
infer, define the public interfaces and namespace object types in the generated
declaration header.

### Custom declaration header and globals

The documented package-level mechanism for aliases, imports, interfaces, and
global augmentation is the napi CLI's custom declaration header:

- configure inline `napi.dtsHeader` in `package.json` when the repository must
  contain no source `.d.ts` file;
- use `napi.dtsHeaderFile` only when a checked-in `.d.ts` header is acceptable;
- `--dts-header` may override the inline header in CI;
- a custom header replaces the default header, so retain the generated comment
  and lint directive when desired.

The inline header may contain the prototype-only interfaces, namespace object
types, and `declare global` block. Global declarations must use unambiguous type
names rather than self-referential declarations such as
`var Integer: typeof Integer` when the same identifier denotes both value and
type.

Do not rely on `NAPI_TYPE_DEF_TMP_FOLDER` or write fragments into napi-rs
temporary directories from `build.rs`. That hook is not documented by the
official type-overwrite or napi-config pages and may change without notice.
`napi_build::setup()` remains the supported `build.rs` setup; declaration header
configuration belongs to the napi CLI configuration.

If an inline header becomes too large, prefer a deterministic Rust or Node build
step that post-processes the generated artifact and is covered by snapshot tests.
Do not disguise that step as a napi-rs-supported merge hook.

## Property and raw-API rules

- Prefer `Reflect.defineProperty` for exact descriptors.
- Constants are non-writable, non-enumerable, and non-configurable.
- Keep constant names and values exact: `MAX_SAFE_INTEGER` is
  `9007199254740991`, not `i64::MAX`.
- APIs absent from the official concept pages must be verified against the
  pinned crate. If unavailable, use the corresponding `napi::sys` operation.
- Check every raw Node-API status and propagate pending JavaScript exceptions.

## Verification

Before finalizing prototype, scalar, Buffer, initialization, or declaration
changes, verify:

- wrapper instances have no own slot properties;
- borrowed methods and accessors reject spoofed receivers;
- namespace and instance prototype chains match the topology above;
- `Object.getPrototypeOf(SmlInt.prototype) === Number.prototype`;
- `Object.getPrototypeOf(SmlNum.prototype) === Number.prototype`;
- `Object.getPrototypeOf(Float.prototype) === Number.prototype`;
- `Object.getPrototypeOf(Integer.prototype) === BigInt.prototype`;
- `Object.getPrototypeOf(BigFlt.prototype) === Object.prototype`;
- `new SmlInt(3) === 3` is false, while `new SmlInt(3) == 3` is true;
- `new Integer(1n) + new SmlNum(1)` throws like `1n + 1`;
- safe-integer checks use `2^53 - 1`;
- `ToInt32` conversions use modulo `2^32`, never saturating casts;
- negative zero is preserved through integer arithmetic fallbacks;
- produced byte values pass the intended Buffer/typed-array brand checks;
- loading in a worker creates and releases a separate `Host`;
- generated declarations match a committed snapshot and pass `tsc --noEmit`;
- `cargo check --all-targets`, runtime tests, Node tests, and Bun tests pass with
  no warnings.

## Numeric compatibility decisions

Do not patch host built-ins. `SmlInt` and `SmlFlt` may use genuine boxed Number
objects so native Number methods can work through `Number.prototype`. Verify
the boxed-object `napi_wrap` path on every supported runtime. `Integer` must
use genuine boxed BigInt objects, while `BigFlt` remains an explicit addon
object. Mixed Number/BigInt arithmetic must throw, matching ECMAScript.

## Runtime cells and public Integer / BigNum

- `Integer` is a separate BigInt-backed global constructor. Its `.cast` and
  `.valueOf()` return BigInt for all widths. `bits: undefined` means unlimited
  precision; `asIntN` and `asUintN` apply explicit width normalization.
- The normal inline cell policy is 64 bits on 64-bit targets and 32 bits on
  32-bit targets; values that do not fit spill into owned heap chunks.
- `SmlInt` is Number-backed at the JavaScript boundary. It stores an i32 on
  64-bit targets and an i16 on 32-bit targets; unsigned instances use the same
  payload width with explicit signedness metadata.
  `Integer` remains the separate arbitrary-precision BigInt constructor.
- `SmlNum` stores f32 on 64-bit targets and binary16 on 32-bit targets. `SmlFlt`
  aliases `SmlNum`; `Float` preserves full f64 precision. These are distinct
  precision policies within the Number family.
- `BigNum` owns exact binary payloads with format metadata. Its `toBuffer()`
  and `toHex()` preserve bits; interpreting float80, binary128 and decimal128
  requires format-specific decoding. Numeric coercion throws until that work
  is implemented.
- The in-memory `ScalarCell` is eight bytes. Inline signed integers fit in
  `[-2^47, 2^47-1]`; larger integers spill into a single contiguous chunk of
  little-endian limbs. Strings, BigNum data and object slots have their own
  contiguous chunks.
- Heap tags contain a 48-bit **arena handle**, never a truncated virtual
  address. User-space pointers are not guaranteed to fit in 48 bits, and
  integer-to-pointer round trips require a sound provenance strategy. A cell
  and any handles copied from it must not outlive the owning `ValueArena`.
- All NaNs are canonicalized at cell ingress. Keep the eight-byte `HeapHeader`
  encoded in the first eight bytes of each chunk and validate kind and bounds
  on every handle lookup.
- Select `ScalarCell` (64-bit tagged values) on 64-bit targets and `CompactCell`
  (32-bit tagged values) on 32-bit targets. `CompactCell` reserves four tag
  bits and has a 28-bit payload; Sml numeric policies are narrower still
  (16-bit small values on 32-bit targets). Full f64 values that do not fit are
  kept exactly in an eight-byte `Number64` chunk. Handles remain owned by the
  `ValueArena` and must not outlive it.

## String runtime representation

- JavaScript strings remain semantically strings (`typeof value === "string"`).
  The runtime's physical representation does not change ECMAScript semantics
  or claim to control V8/JSC/Bun's internal string layout.
- `value::string::StringTable` owns runtime `StringLiteralId`s and materialized
  `StringBlock`s. An AOT compiler may emit referenced literals into a
  read-only pool and remove unreferenced entries before runtime initialization;
  that optimization is not yet wired to this prototype table. Literal IDs are
  handles into the owning table and must not outlive it.
- A string literal is still a JavaScript string (`typeof === "string"`), never
  a JavaScript Symbol. String equality is by contents; Symbol equality is by
  unique identity. They may share an ID allocator or handle-table machinery,
  but must keep separate tags, conversion rules, and property-key behavior.
- A `StringBlock` is one immutable contiguous allocation containing a 16-byte
  metadata prefix and the encoded data. Its encoding is ASCII, UTF-8, or
  little-endian UTF-16. The prefix records byte length, JavaScript UTF-16 code
  unit length, and a hash computed over UTF-16 code units, so equivalent UTF-8
  and UTF-16 strings compare and hash consistently.
- UTF-16 blocks preserve unpaired surrogates, as JavaScript strings require.
  JavaScript-style slicing uses UTF-16 code-unit indices and may therefore
  produce a block containing an unpaired surrogate. `to_utf8_lossy` replaces
  unpaired surrogates; it is explicitly lossy.
- `StringTable::concat` and `StringTable::slice` materialize contiguous blocks.
  Explicit block constructors also materialize short strings. ASCII/UTF-8 data
  can be borrowed as Rust `&str`; UTF-16 data needs decoding for Rust string
  APIs. Do not promise zero-copy conversion to/from NAPI JavaScript strings,
  Node Buffers, or host-engine storage.
- `ScalarCell` and `CompactCell` distinguish literal and block handles in
  their string tags. These payloads are arena/table handles, never truncated
  raw pointers. On 32-bit targets literal IDs use `u16`; on wider targets they
  use `u32`. A handle is valid only with the table that allocated it.
- String blocks, bytes, BigInt limbs, BigNum payloads, arrays, and objects are
  composite data. Null, undefined, booleans, and fitting numeric values use
  immediate cell representations. Semantic type and physical storage tier are
  separate classifications.
- Do not patch the host `String` constructor or `String.prototype`. Any future
  addon `String` API must be additive and preserve built-in callable and
  primitive behavior.
