# alloy-engine

Pure Rust ECMAScript runtime for Alloy TS (MVP subset). This crate owns values,
Realms, the lexer, parser and evaluator. It has no napi or Node dependency and
contains no `unsafe` code.

The NAPI `Engine` / `Context` facade that talks to a host JavaScript process
lives in [alloy-ts/core](https://dev.azure.com/ChenSoftware/alloy-ts/_git/core).

## Layout

```text
RuntimeContext (lifecycle)
  └─ Realm ── global bindings
        │   └─ Heap (UTF-8 strings)
        └─ ScriptRecord ──► evaluator ──► Completion<Value>
```

See [`docs/architecture.md`](docs/architecture.md),
[`docs/engine-plan.md`](docs/engine-plan.md) for the language subset, and
[`docs/api`](docs/api) for the public Rust API.

## Development

```sh
cargo fmt --check
cargo check --all-targets
cargo test
```
