# Node-API compatibility policy

## Initial target

- Stable Node-API version: 8
- Experimental APIs: unsupported
- Legacy `napi_module_register`: unsupported in the first loader
- Arbitrary third-party `.node` loading: not claimed during controlled-fixture
  phases
- Native-library unload: disabled before process termination

The machine-readable status source is
[`node-api-abi-manifest.yaml`](node-api-abi-manifest.yaml).

## Status vocabulary

| Status            | Meaning                                                          |
| ----------------- | ---------------------------------------------------------------- |
| `designed`        | Semantics and ownership are documented; no implementation exists |
| `blocked_runtime` | Requires an Alloy runtime feature such as objects or functions   |
| `blocked_loader`  | Requires controlled native symbol routing/loading                |
| `blocked_async`   | Requires worker/owner-thread scheduling                          |
| `unsupported`     | Intentionally unavailable for the selected target                |
| `implemented`     | Implementation and ABI fixture both pass                         |

No symbol is marked `implemented` solely because it is exported.

## API-family boundaries

- Core APIs cover engine-neutral values, objects, functions, scopes,
  exceptions, references, promises, and typed data.
- Node extensions include Buffer-specific behavior and Node integration hooks.
- Async APIs require worker and owner-thread lifecycle semantics.
- libuv APIs require a real compatible loop and separate ABI validation.

## Conformance progression

1. ABI-level Rust tests for environment and handle validation.
2. Controlled C addon linked to the Alloy bridge.
3. The same fixture under Node to establish expected behavior.
4. Applicable `js-native-api` tests.
5. Applicable Node `node-api` tests.
6. Representative third-party packages and package-loader behavior.

Every skipped upstream test records the symbol, reason, required runtime
milestone, and issue/owner.

## Compatibility claims

Compatibility is reported per Node-API version and platform. “Supports
Node-API v8” requires all applicable v8 manifest entries and conformance tests,
not merely successful module initialization.
