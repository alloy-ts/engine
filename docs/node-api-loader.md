# Node-API loader and symbol routing

## Problem

Loading a `.node` library is not sufficient. Its unresolved `napi_*` imports
must bind to the Alloy host associated with the embedded Realm. When Alloy is
itself running inside Node or Bun, ambient process lookup can incorrectly bind
the embedded addon to the outer host.

The loader must therefore prove symbol ownership rather than depending on
ordinary `dlopen`/`LoadLibrary` behavior.

## Supported topologies

### Standalone Alloy process

Alloy can export a Node-API bridge from the executable or a dedicated shared
library. Even here, export visibility and the addon's expected import names
must be tested per platform.

### Nested Alloy Engine

`@alloy-ts/engine` is loaded by an outer host whose process already exports
Node-API. A second addon loaded normally may resolve against that outer ABI.
This topology requires isolation or explicit import routing.

## Platform analysis gate

| Platform  | Risk                                                                       | Candidate controlled strategy                                                               |
| --------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| ELF/Linux | Global symbol interposition binds `napi_*` to the outer host               | Dedicated namespace/bridge where available, or an Alloy-owned relocation loader             |
| macOS     | Two-level namespace and dynamic lookup behavior depend on addon link flags | Explicit bridge identity or controlled relocation; never assume flat lookup selects Alloy   |
| Windows   | Addons commonly import from `node.exe` through a Node import library       | Compatible bridge/import strategy, import-table rewriting, custom loader, or helper process |

No strategy is accepted without a native fixture proving which implementation
each imported function reaches.

## Initial controlled fixture

The first fixture may link explicitly against an Alloy bridge library. This
validates the ABI and environment semantics but is labelled “controlled Alloy
addon,” not arbitrary `.node` compatibility.

The fixture exports `napi_register_module_v1` and optionally
`node_api_module_get_api_version_v1`. It records a bridge-specific diagnostic
value so tests can prove calls reached Alloy rather than the outer Node host.

## Registration algorithm

1. Validate path, platform, architecture, and configured loading topology.
2. Open the library in a routing mode that cannot silently select ambient
   Node-API symbols.
3. Resolve `node_api_module_get_api_version_v1` when present.
4. Reject unsupported or experimental versions before module initialization.
5. Resolve `napi_register_module_v1` with the platform calling convention.
6. Create a module environment and rooted exports object.
7. Invoke registration on the Engine owner thread.
8. Fail if an exception remains pending.
9. Retain the library for at least as long as any callback, finalizer, wrapped
   object, external buffer, reference, or async work can reach its code/data.

## Unloading policy

The first loader does not unload native libraries before process termination.
Environment disposal still releases Alloy handles and runs eligible finalizers,
but keeping the code image mapped avoids calling stale function pointers.

Unload support is a later opt-in capability requiring proof that all callbacks,
references, finalizers, externals, cleanup hooks, thread-safe functions, and
async work owned by the image are gone.

## Rejected shortcuts

- relying on the outer Node process's exported `napi_*` symbols;
- returning a fake `uv_loop_t`;
- casting CompactCell or an object pointer to `napi_value`;
- loading an arbitrary addon and inferring compatibility from initialization;
- describing explicitly bridge-linked fixtures as general `.node` support.
