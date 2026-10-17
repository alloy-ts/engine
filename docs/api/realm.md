# Realm API Documentation and NAPI Bindings

## Overview
In ECMAScript engines (such as Boa or QuickJS), a **Realm Record** (or Realm) represents an execution environment with its own set of intrinsic objects, global object, global scope/environment, and loaded modules.

## Boa Realm vs. NAPI JS Bindings

### Boa `Realm` Structure
Boa defines `Realm` as a garbage-collected wrapper (`Gc<Inner>`) around:
- `intrinsics`: Core built-in object prototypes and constructors (`Object`, `Array`, `Function`, `Error`, etc.).
- `environment`: Global `DeclarativeEnvironment`.
- `scope`: Scope tree associated with the global declarative environment.
- `global_object`: The realm's global JavaScript object (`JsObject`).
- `global_this`: The realm's global `this` binding (`JsObject`).
- `template_map`: Cache for template literal objects.
- `loaded_modules`: Map of loaded ECMAScript modules.
- `host_classes`: Registry of host-registered classes (`TypeId` -> `StandardConstructor`).
- `host_defined`: User/Host data attached to the realm.

### NAPI / JavaScript Surface
In our NAPI host bindings:
- **Execution Context:** `Context` wraps a realm instance in QuickJS/rquickjs (`QjsContext`).
- **Globals & Evaluation:** Calling `ctx.eval(...)` or `ctx.with(...)` operates within the realm's global scope and intrinsic set.
- **Module Support:** `eval_module` handles module declaration and evaluation scoped to the context realm.
- **Garbage Collection:** Garbage collection is coordinated via `ctx.run_gc()`.
