# Engine

This document describes the JavaScript-facing `Engine` API exported via NAPI by `@alloy-ts/engine`, backed by [Boa Engine (`boa_engine`)](https://crates.io/crates/boa_engine).

## Overview

The `Engine` object represents an embedded ECMAScript runtime instance. It wraps an underlying `boa_engine::Context` execution environment and manages isolated contexts (`Context`), runtime limits, garbage collection, and job draining.

## Example Usage

```ts
import { Engine } from "@alloy-ts/engine";

// Instantiate the Boa engine (default backend: "boa")
const engine = new Engine(); // or Engine.boa()

// Evaluate JavaScript expressions
const result = engine.eval("let two = 1 + 1; two + '2'");
console.log(result); // "22"

// Create isolated contexts
const ctx = engine.createContext();
ctx.eval("console.log(3 + 3.33333)"); // prints "6.33333"

// Tuning & GC
engine.setRuntimeLimits({ stackSize: 512 * 1024 });
engine.runGc();
```

## API Surface

| JS Method | Return / Type | Description |
|---|---|---|
| `new Engine(backend?: string)` | `Engine` | Construct new Engine instance (defaults to `"boa"`, options `"boa"` \| `"quickjs"`) |
| `Engine.boa()` | `Engine` | Static factory for Boa-backed engine |
| `Engine.quickjs()` | `Engine` | Static factory for QuickJS engine |
| `engine.backend` | `string` | Getter for engine backend name (`"boa"`) |
| `engine.createContext()` | `Context` | Creates a new isolated `Context` |
| `engine.eval(source)` | `JsonValue` | Evaluates JS code string in primary context |
| `engine.runJobs()` | `()` | Runs microtask promise job queue |
| `engine.runGc()` | `()` | Triggers force garbage collection sweep (`boa_engine::gc::force_collect`) |
| `engine.memoryUsage()` | `JsonValue` | Returns engine allocation and memory stats |
| `engine.runtimeLimits()` | `RuntimeLimitsView` | Get stack/loop/backtrace limits |
| `engine.setRuntimeLimits(limits)` | `()` | Set stack/loop/backtrace limits |
| `engine.optimizerOptions()` | `OptimizerOptionsView` | Get AST optimizer option flags |
| `engine.setOptimizerOptions(opts)` | `()` | Set AST optimizer option flags |
| `engine.isDisposed()` | `boolean` | Checks if host environment has been unloaded |
| `engine.weak()` | `JsWeakEngine` | Obtains weak handle to the engine |

## Parity with Boa Engine

`boa_engine` provides an ECMAScript VM with Bytecompiler, GC, AST Optimizer, and Execution Context. `Engine` exposes a safe, thread-safe NAPI binding to this runtime in Node.js / JavaScript.
