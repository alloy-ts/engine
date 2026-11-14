# Embedded Node-API host

## Status

Phase 0 design is complete. Phase 1 now has a safe Rust substrate for
environment ownership, local handles and scopes, primitive Realm values,
value-type inspection, and pending exceptions. It deliberately does not export
Node-API symbols, load native libraries, add `napi_sym`, or claim `.node`
compatibility.

The outer `@alloy-ts/engine` addon and the embedded host are different roles:

```mermaid
flowchart LR
  Outer[Node or Bun] -->|host Node-API| Package[@alloy-ts/engine]
  Package --> Engine
  Engine --> Realm[Alloy Realm]
  Native[Embedded native addon] -->|Alloy Node-API ABI| Host[NodeApiHost]
  Host --> Realm
```

NAPI-RS remains the outer addon facade. The embedded host owns its own
environment, handles, pending-exception state, callbacks, and lifecycle.

## Header and version policy

The initial compatibility target is stable Node-API version 8. The exact ABI
declarations must be generated or verified from a vendored, checksummed copy
of the official Node headers selected by a later implementation PR. The current
development dependency contains an upstream-compatible header snapshot whose
default `NAPI_VERSION` is 8; it is a planning reference, not the permanent ABI
source of truth.

Experimental APIs are excluded. Symbols introduced after version 8 are listed
as unavailable until the target version advances. The addon may export
`node_api_module_get_api_version_v1`; absence means the loader applies the
documented default version policy.

## Module registration ABI

The official header model uses opaque pointer types and the C calling
convention (`__cdecl` on Windows):

```c
typedef struct napi_env__* napi_env;
typedef struct napi_value__* napi_value;
typedef struct napi_callback_info__* napi_callback_info;

typedef napi_value (NAPI_CDECL* napi_addon_register_func)(
    napi_env env,
    napi_value exports);

typedef int32_t (NAPI_CDECL* node_api_addon_get_api_version_func)(void);
```

For native modules, the loader resolves:

- `napi_register_module_v1`, with `napi_addon_register_func` semantics;
- optionally `node_api_module_get_api_version_v1`, returning the requested
  Node-API version.

The loader creates a rooted exports object in the module environment, invokes
the registration function on the Engine owner thread, and uses its non-null
return value as exports. A null return is accepted only when no exception is
pending and means the original exports object. A pending exception makes
initialization fail and preserves the thrown Alloy value for the caller.

Legacy `napi_module_register` is a separate, deprecated ABI family and is not
part of the first implementation slice.

## ABI ownership

Every symbol is owned by one subsystem and described in
[`node-api-abi-manifest.yaml`](node-api-abi-manifest.yaml). Required fields
include exact signature, header/version origin, ABI family, platform calling
convention, thread rule, handle-rooting rule, pending-exception behavior,
runtime prerequisites, fixture, and status.

ABI families are:

- `core`: engine-agnostic Node-API values, objects, functions, scopes, errors;
- `registration`: module entry points and version negotiation;
- `node_extension`: Node-specific APIs outside the engine-neutral core;
- `buffer_extension`: Node Buffer behavior;
- `async`: async work and thread-safe functions;
- `libuv`: APIs requiring a real ABI-compatible `uv_loop_t`;
- `legacy`: deprecated registration or compatibility paths;
- `experimental`: excluded from the stable target.

## Environment and handle model

An environment is bound to one Engine, one Realm, one owner thread, and one
loaded module registration. It contains:

- current lifecycle state;
- pending exception;
- handle-scope stack;
- local value-handle table;
- persistent and weak reference tables;
- wrapped-native and external-value registries;
- finalizer and cleanup queues;
- module/library ownership.

`napi_value` is never a pointer to CompactCell or movable object storage. It is
an opaque entry validated against environment identity, slot generation, scope
state, and shutdown state.

| Handle kind            | Rooting                                      | Lifetime                                      |
| ---------------------- | -------------------------------------------- | --------------------------------------------- |
| Local `napi_value`     | Current handle scope                         | Until its scope closes                        |
| Escaped local value    | Parent scope                                 | Until parent scope closes                     |
| Strong `napi_ref`      | Strong while reference count is non-zero     | Until explicit deletion or environment close  |
| Weak `napi_ref`        | Not a strong root                            | Cleared during collector weak processing      |
| Wrapped native storage | Object plus environment finalizer registry   | Remove-wrap, object finalization, or shutdown |
| External value         | Environment and collector/finalizer contract | Until collection or environment shutdown      |

Cross-environment use returns `napi_invalid_arg`; stale generations and closed
scope handles never resolve to reused values.

## Pending exceptions

The environment owns one pending thrown Alloy value. Each manifest entry says
whether it may run with a pending exception and whether failure sets one.
Unless explicitly permitted, an API called with a pending exception returns
`napi_pending_exception` without mutating output parameters.

The first exception slice includes creating Error variants, throwing arbitrary
Alloy values, checking pending state, and retrieving/clearing the pending
exception. Native callback boundaries translate abrupt Alloy completions into
this state.

## First executable ABI slice

The first code-bearing PR after Phase 0 should remain smaller than a loader:

1. Environment state and status/error reporting.
2. Local handle scopes with generation checks.
3. Undefined, null, booleans, numbers, UTF-8/UTF-16 strings, and `napi_typeof`.
4. Pending-exception operations.
5. ABI-level unit tests that do not load a shared library.

Objects, callbacks, module registration, and the controlled native fixture
follow only when the Alloy object/function milestones provide the necessary
semantics.

### Phase 1 implementation status

`src/node_api_host/mod.rs` implements the safe operations that a later C ABI
layer will call. Its handles carry environment identity, slot index, and
generation; they are not raw pointers and never point at Realm storage. Scope
closure invalidates local handles before slots can be reused. Environments
also enforce owner-thread, pending-exception, and closing-state rules.

UTF-8 and scalar-valid UTF-16 inputs allocate ordinary Realm strings. A UTF-16
input containing a lone surrogate returns `Status::StringEncoding`, preserving
the documented MVP limitation rather than replacing data. No manifest entry is
marked as exported yet: the C declarations, pointer validation, and platform
symbol visibility remain a later ABI-layer milestone.

`ValueType::Object` currently classifies only the Realm's temporary native
error handles. General objects and functions wait for the object-model
milestone; BigInt and Symbol wait for their runtime value representations.
Phase 1 therefore does not imply support for those Node-API value categories.

## Phase 0 acceptance gate

Implementation starts only after agreement on:

- the official header snapshot and version-8 policy;
- standalone versus nested loading topology;
- non-ambient symbol routing;
- opaque-handle validation and rooting;
- pending-exception rules;
- shutdown and non-unload policy;
- initial native fixture and expected Node comparison.
