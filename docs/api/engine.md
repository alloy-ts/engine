# `Engine`

```ts
import { Engine } from "repository";

const engine = new Engine();
const context = engine.createContext();
```

## Implemented (MVP)

- `new Engine()` creates an engine. It holds no runtime state.
- `engine.createContext(): Context` returns a new [Context](context.md) with
  its own Realm, global bindings and heap. Contexts from the same or different
  engines are fully isolated.
- `Engine` and `Context` are module exports only; loading the addon does not
  add them to `globalThis`.

## Architectural invariants

- An Engine never shares bindings or strings between its contexts.
- Creating or using contexts never touches the host `globalThis`.

## Planned post-MVP work

- Engine-level configuration (limits, host hooks) once the runtime needs it.
