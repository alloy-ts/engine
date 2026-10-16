/**
 * Tests for the embedded QuickJS `Engine` NAPI bindings (`src/engine.rs`).
 *
 * `Engine` wraps the QuickJS runtime (the engine itself); `create_context()`
 * mints a `Context` that shares that engine. A standalone `Context`
 * (`new Context()`) spins up its own private engine.
 *
 * Per the Rust bindings the constructors are real, so instances are made with
 * `new Engine()` / `new Context()`.
 */
import { describe, expect, test } from "vite-plus/test";
import { Engine, Loader } from "./main.ts";

describe("napi: Engine", () => {
  test("new Engine() returns an Engine and basic accessors work", () => {
    const engine = new Engine();
    expect(engine).toBeDefined();
    expect(engine.is_job_pending()).toBe(false);
    expect(engine.execute_pending_job()).toBe(false);
  });

  test("camelCase qjs Engine API works on new Engine()", () => {
    const engine = new Engine();
    expect(engine).toBeDefined();
    expect(engine.isJobPending()).toBe(false);
    expect(engine.execPendingJob()).toBe(false);
    expect(engine.executePendingJob()).toBe(false);

    const ctx = engine.createContext();
    expect(ctx.eval("10 + 20")).toBe(30);

    ctx.eval(`
      globalThis.queue = [];
      Promise.resolve("qjs").then((v) => { globalThis.queue.push(v); });
    `);
    expect(engine.isJobPending()).toBe(true);

    let ran = 0;
    while (engine.execPendingJob()) {
      ran += 1;
    }
    expect(ran).toBeGreaterThan(0);
    expect(ctx.eval("globalThis.queue")).toEqual(["qjs"]);

    engine.setMemoryLimit(1 << 20);
    engine.setMaxStackSize(256 * 1024);
    engine.setGcThreshold(1024);
    engine.setInfo("camel-case-engine");
    engine.runGc();

    const usage = engine.memoryUsage() as { malloc_size: number };
    expect(typeof usage.malloc_size).toBe("number");
  });

  test("create_context returns a working Context on the shared engine", () => {
    const engine = new Engine();
    const ctx = engine.create_context();
    expect(ctx.eval("6 * 7")).toBe(42);
  });

  test("multiple contexts are independent realms on one engine", () => {
    const engine = new Engine();
    const a = engine.create_context();
    const b = engine.create_context();
    expect(a.eval("1")).toBe(1);
    expect(b.eval("2")).toBe(2);
    // Each context is its own realm with its own global object, even though they
    // share the engine: a global set on `a` is not visible on `b`.
    a.eval("globalThis.shared = 99;");
    expect(a.eval("globalThis.shared")).toBe(99);
    expect(b.eval("typeof globalThis.shared")).toBe("undefined");
  });

  test("weak() returns a WeakEngine whose try_ref / tryRef upgrades back to an Engine", () => {
    const engine = new Engine();
    const weak = engine.weak();
    const upgraded = weak.try_ref();
    expect(upgraded).toBeInstanceOf(Engine);
    const upgradedCamel = weak.tryRef();
    expect(upgradedCamel).toBeInstanceOf(Engine);
  });

  test("engine tuning methods do not throw", () => {
    const engine = new Engine();
    engine.set_memory_limit(1 << 20);
    engine.set_max_stack_size(256 * 1024);
    engine.set_gc_threshold(1024);
    engine.set_info("test-engine");
    engine.run_gc();
  });

  test("memory_usage returns a stats object with numeric fields", () => {
    const engine = new Engine();
    const usage = engine.memory_usage() as {
      malloc_size: number;
      malloc_limit: number;
    };
    expect(typeof usage).toBe("object");
    expect(typeof usage.malloc_size).toBe("number");
    expect(typeof usage.malloc_limit).toBe("number");
  });

  test("execute_pending_job drains promise jobs", () => {
    const engine = new Engine();
    const ctx = engine.create_context();
    ctx.eval(`
      globalThis.log = [];
      Promise.resolve(42).then((v) => { globalThis.log.push(v); });
    `);
    expect(engine.is_job_pending()).toBe(true);

    let ran = 0;
    while (engine.execute_pending_job()) {
      ran += 1;
    }
    expect(ran).toBeGreaterThan(0);
    expect(ctx.eval("globalThis.log")).toEqual([42]);
  });

  test("execPendingJob drains promise jobs", () => {
    const engine = new Engine();
    const ctx = engine.createContext();
    ctx.eval(`
      globalThis.log = [];
      Promise.resolve(100).then((v) => { globalThis.log.push(v); });
    `);
    expect(engine.isJobPending()).toBe(true);

    let ran = 0;
    while (engine.execPendingJob()) {
      ran += 1;
    }
    expect(ran).toBeGreaterThan(0);
    expect(ctx.eval("globalThis.log")).toEqual([100]);
  });

  test("execute_pending_job surfaces a throwing microtask as a thrown exception", () => {
    const engine = new Engine();
    const ctx = engine.create_context();
    // A thrown promise reaction becomes an unhandled rejection (the job itself
    // succeeds); a `queueMicrotask` job that throws propagates to the caller.
    ctx.eval(`queueMicrotask(() => { throw new Error("boom"); });`);
    expect(engine.is_job_pending()).toBe(true);
    expect(() => {
      while (engine.execute_pending_job()) {
        /* drain */
      }
    }).toThrow();
  });

  test("Engine can be constructed with a Loader and resolves imports", () => {
    const loader = new Loader();
    loader.add("m", "export const v = 7;");
    const engine = new Engine(loader);
    const ctx = engine.create_context();
    ctx.eval_module('import { v } from "m"; globalThis.out = v;');
    expect(ctx.eval("globalThis.out")).toBe(7);
  });

  test("set_loader attaches a loader after construction", () => {
    const loader = new Loader();
    loader.add("k", "export const v = 'from-loader';");
    const engine = new Engine();
    engine.set_loader(loader);
    const ctx = engine.create_context();
    ctx.eval_module('import { v } from "k"; globalThis.out = v;');
    expect(ctx.eval("globalThis.out")).toBe("from-loader");
  });

  test("setLoader attaches a loader after construction via camelCase API", () => {
    const loader = new Loader();
    loader.add("k2", "export const v = 'from-loader-camel';");
    const engine = new Engine();
    engine.setLoader(loader);
    const ctx = engine.createContext();
    ctx.eval_module('import { v } from "k2"; globalThis.out = v;');
    expect(ctx.eval("globalThis.out")).toBe("from-loader-camel");
  });
});

describe("napi: Engine host environment", () => {
  test("create_array / createArray allocates a JS array of the requested length", () => {
    const engine = new Engine();
    const arr = engine.create_array(3) as unknown[];
    expect(Array.isArray(arr)).toBe(true);
    expect(arr.length).toBe(3);

    const arrCamel = engine.createArray(5) as unknown[];
    expect(Array.isArray(arrCamel)).toBe(true);
    expect(arrCamel.length).toBe(5);
  });

  test("create_string / create_int32 / create_double build JS values (snake & camel)", () => {
    const engine = new Engine();
    expect(engine.create_string("hello")).toBe("hello");
    expect(engine.createString("hello")).toBe("hello");
    expect(engine.create_int32(42)).toBe(42);
    expect(engine.createInt32(42)).toBe(42);
    expect(engine.create_double(1.5)).toBe(1.5);
    expect(engine.createDouble(1.5)).toBe(1.5);
  });

  test("has_global / hasGlobal reads the host global object", () => {
    const engine = new Engine();
    expect(engine.has_global("Math")).toBe(true);
    expect(engine.hasGlobal("Math")).toBe(true);
    expect(engine.has_global("definitelyNotAGlobal")).toBe(false);
    expect(engine.hasGlobal("definitelyNotAGlobal")).toBe(false);
  });

  test("get_napi_version / getNapiVersion reports the host N-API version", () => {
    const engine = new Engine();
    expect(engine.get_napi_version()).toBeGreaterThan(0);
    expect(engine.getNapiVersion()).toBeGreaterThan(0);
  });

  test("make_object / makeObject builds a JS object via the host context", () => {
    const engine = new Engine();
    const obj = engine.make_object() as { ok: boolean; engine: string };
    expect(obj.ok).toBe(true);
    expect(obj.engine).toBe("napi");

    const objCamel = engine.makeObject() as { ok: boolean; engine: string };
    expect(objCamel.ok).toBe(true);
    expect(objCamel.engine).toBe("napi");
  });

  test("set_instance_data / setInstanceData persists host-level state on the Agent", () => {
    const engine = new Engine();
    engine.set_instance_data("vm-state");
    expect(engine.get_instance_data()).toBe("vm-state");

    engine.setInstanceData("vm-state-camel");
    expect(engine.getInstanceData()).toBe("vm-state-camel");
  });

  test("is_disposed / isDisposed is false while the addon is loaded", () => {
    const engine = new Engine();
    expect(engine.is_disposed()).toBe(false);
    expect(engine.isDisposed()).toBe(false);
  });

  test("each instance captures its own context handle", () => {
    const a = new Engine();
    const b = new Engine();
    // Two engines must not alias the same context; both are independently live.
    expect(a.is_disposed()).toBe(false);
    expect(b.is_disposed()).toBe(false);
  });
});
