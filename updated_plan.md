1. *Initialize CMake build system* with ThinLTO and feature flags (ALLOY_ENABLE_WEB, ALLOY_ENABLE_WASI, ALLOY_ENABLE_ISOLATION).
2. *Implement the Capability-Based Security Model* in `src/core/capability.c`.
3. *Implement Native Host mappings for WASI Preview 2 (WIT)* in `src/core/wasi/`.
4. *Upgrade alloy:web IPC* in `src/core/ipc.c` with X25519 handshake and binary serialization.
5. *Establish alloy:gui platform abstraction* in `src/core/gui/`.
6. *Implement Shim Injection via Plugin Hooks*:
   - Update `BuildPlugin` and `PluginBuild` in `src/core/build.ts` to support `onInit`, `onParse`, and `beforeBundle`.
   - Implement `ShimPlugin` that uses these hooks to inject mandatory async/await and Web API shims into the code during the build process, rather than hardcoding injections in the transpiler.
   - Update `ARCHITECTURE.md` and docs to reflect plugin-based shim injection.
7. *Integrate WASI and GUI bindings into MicroQuickJS*.
8. *Memory Management and Leak Prevention* in the C layer.
9. *Run all unit and property-based tests* to ensure the entire implementation meets the requirements.
10. *Complete pre-commit steps* to ensure proper testing, verification, review, and reflection are done.
11. *Submit the change.*
