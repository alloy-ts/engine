# Runtime roadmap

## Direction

Alloy owns the ECMAScript value representation, object model, garbage
collector, evaluator, built-ins, Realms and execution contexts. This allows
future runtime storage such as CompactCell and literal interning to be designed
as part of the engine rather than layered over another engine's private values.

`@alloy-ts/core` remains the value layer. `@alloy-ts/engine` owns script
execution, its NAPI facade and CLI, and the future Node-API host. Creating an
Engine or Context never modifies the embedding host's globals. The embedded
Realm exposes standard ECMAScript globals with standard behavior; Alloy
extensions use documented, generic APIs rather than a single `Alloy` global.

Boa, QuickJS and other engines may be used as differential-testing oracles or
optional experiments. Their private value layouts and engine-owned collectors
cannot make CompactCell the storage representation of those engines. Alloy's
production runtime instead pairs CompactCell with an Alloy-owned collector
designed to trace its tags and references, manage its roots, and implement its
movement, weak-reference and finalization rules.

## Milestones

1. Complete the primitive evaluator and statement semantics against ECMA-262,
   retaining bounded parsing and specification-shaped Completion Records.
2. Add Alloy-owned objects, arrays, functions, closures, prototypes, built-ins,
   execution-context stacks, and a tracing collector. Extend host conversion
   only as these runtime values become representable. The detailed invariants
   and milestone interfaces are in [object-model.md](object-model.md).
3. Add promises, jobs and modules, followed by incremental Node-compatible
   JavaScript modules and package resolution.
4. Implement a separate Node-API host subsystem: ABI symbols, environments,
   handles and scopes, callbacks, references, exceptions, finalizers, buffers,
   typed arrays, BigInt, dynamic `.node` loading, and conformance tests using
   native test addons. NAPI-RS remains the outer addon facade; it is not the
   embedded host implementation.
5. Add async work with an Alloy-owned worker pool and owner-thread completion
   queue. Add libuv interoperability only when a real compatible `uv_loop_t` is
   supplied. Evaluate `napi_sym` when an explicit Node-API ABI manifest exists;
   exporting symbols alone does not implement Node-API behavior.
6. Explore encrypted-MMU or protected-memory storage only after ordinary GC,
   rooting, finalization, threading, and shutdown semantics are stable.

ECMA-262 is the semantic authority. Bun and Deno are implementation references
for Node compatibility, not dependencies or substitutes for the Alloy runtime.

The MVP string heap stores UTF-8 Unicode scalar values. Host JavaScript strings
containing lone UTF-16 surrogates are rejected at the boundary rather than
silently replaced. Full ECMAScript String compatibility requires migrating the
runtime representation to UTF-16 code units or WTF-8 so lone surrogates are
preserved exactly.
