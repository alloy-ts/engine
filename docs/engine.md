# Engine API

The `Engine` class provides access to N-API environment operations as well as QuickJS runtime execution controls.

## Usage

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();

// N-API helper functions
engine.createString("hello");
engine.createInt32(42);
engine.createArray(3);
engine.hasGlobal("Math");
engine.getNapiVersion();

// QuickJS runtime controls
engine.isJobPending();
engine.execPendingJob();
engine.runGc();
engine.setMemoryLimit(1024 * 1024);
```
