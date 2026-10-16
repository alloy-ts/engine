---
title: 'Engine'
description: JavaScript engine context and QuickJS runtime execution bindings.
---

# Engine

The `Engine` class represents both the Node-API context handle (`napi::Env`) and an embedded QuickJS runtime execution engine (`rquickjs::Runtime`).

## Usage

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();

// Context value creation
engine.create_string("hello");
engine.create_int32(42);
engine.create_double(3.14);
engine.create_array(5);
engine.create_object();

// Global inspection & versions
engine.has_global("Math");
engine.get_napi_version();

// Instance data persistence
engine.set_instance_data("state-data");
engine.get_instance_data();

// QuickJS runtime controls
engine.set_memory_limit(1024 * 1024);
engine.set_max_stack_size(256 * 1024);
engine.set_gc_threshold(512 * 1024);
engine.is_job_pending();
engine.execPendingJob();
engine.run_gc();

// Synchronous error throwing
engine.throw_error("Custom error message", "ERR_CUSTOM");
```
