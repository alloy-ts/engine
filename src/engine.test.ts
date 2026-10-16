/**
 * Tests for the embedded QuickJS `Engine` NAPI bindings (`src/engine.rs`).
 *
 * `Engine` wraps BOTH:
 * - the QuickJS runtime (the engine itself)
 * - the host N-API environment (`napi::Env`), captured at construction.
 *
 * Both camelCase and snake_case API aliases are supported on `Engine`.
 */
import { describe, expect, test } from "vite-plus/test";
import { Engine, Loader } from "./main.ts";

describe("napi: Engine (QuickJS runtime API)", () => {
  test("new Engine() returns an Engine and basic accessors work", () => {
    const engine = new Engine();
    expect(engine).toBeDefined();
    expect(engine.is_job_pending()).toBe(false);
    expect(engine.isJobPending()).toBe(false);
    expect(engine.execute_pending_job()).toBe(false);
    expect(engine.executePendingJob()).toBe(false);
    expect(engine.exec_pending_job()).toBe(false);
    expect(engine.execPendingJob()).toBe(false);
  });

  test("create_context / createContext returns a working Context on the shared engine", () => {
    const engine = new Engine();
    const ctx1 = engine.create_context();
    expect(ctx1.eval("6 * 7")).toBe(42);

    const ctx2 = engine.createContext();
    expect(ctx2.eval("10 + 20")).toBe(30);
  });

  test("multiple contexts are independent realms on one engine", () => {
    const engine = new Engine();
    const a = engine.create_context();
    const b = engine.createContext();
    expect(a.eval("1")).toBe(1);
    expect(b.eval("2")).toBe(2);
    // Each context is its own realm with its own global object
    a.eval("globalThis.shared = 99;");
    expect(a.eval("globalThis.shared")).toBe(99);
    expect(b.eval("typeof globalThis.shared")).toBe("undefined");
  });

  test("weak() returns a WeakEngine whose try_ref / tryRef upgrades back to an Engine", () => {
    const engine = new Engine();
    const weak = engine.weak();
    const upgraded = weak.try_ref();
    expect(upgraded).toBeInstanceOf(Engine);

    const upgraded2 = weak.tryRef();
    expect(upgraded2).toBeInstanceOf(Engine);
  });

  test("engine tuning methods (both snake_case and camelCase) do not throw", () => {
    const engine = new Engine();
    engine.set_memory_limit(1 << 20);
    engine.setMemoryLimit(1 << 20);
    engine.set_max_stack_size(256 * 1024);
    engine.setMaxStackSize(256 * 1024);
    engine.set_gc_threshold(1024);
    engine.setGcThreshold(1024);
    engine.set_info("test-engine");
    engine.setInfo("test-engine-camel");
    engine.run_gc();
    engine.runGc();
  });

  test("memory_usage / memoryUsage returns a stats object with numeric fields", () => {
    const engine = new Engine();
    const usage = engine.memory_usage() as {
      malloc_size: number;
      malloc_limit: number;
    };
    expect(typeof usage).toBe("object");
    expect(typeof usage.malloc_size).toBe("number");
    expect(typeof usage.malloc_limit).toBe("number");

    const usageCamel = engine.memoryUsage() as {
      malloc_size: number;
      malloc_limit: number;
    };
    expect(typeof usageCamel).toBe("object");
    expect(typeof usageCamel.malloc_size).toBe("number");
  });

  test("execPendingJob / executePendingJob drains promise jobs", () => {
    const engine = new Engine();
    const ctx = engine.createContext();
    ctx.eval(`
      globalThis.log = [];
      Promise.resolve(42).then((v) => { globalThis.log.push(v); });
    `);
    expect(engine.isJobPending()).toBe(true);

    let ran = 0;
    while (engine.execPendingJob()) {
      ran += 1;
    }
    expect(ran).toBeGreaterThan(0);
    expect(ctx.eval("globalThis.log")).toEqual([42]);
  });

  test("executePendingJob surfaces a throwing microtask as a thrown exception", () => {
    const engine = new Engine();
    const ctx = engine.createContext();
    ctx.eval(`queueMicrotask(() => { throw new Error("boom"); });`);
    expect(engine.isJobPending()).toBe(true);
    expect(() => {
      while (engine.executePendingJob()) {
        /* drain */
      }
    }).toThrow();
  });

  test("Engine can be constructed with a Loader and resolves imports", () => {
    const loader = new Loader();
    loader.add("m", "export const v = 7;");
    const engine = new Engine(loader);
    const ctx = engine.createContext();
    ctx.eval_module('import { v } from "m"; globalThis.out = v;');
    expect(ctx.eval("globalThis.out")).toBe(7);
  });

  test("set_loader / setLoader attaches a loader after construction", () => {
    const loader = new Loader();
    loader.add("k", "export const v = 'from-loader';");
    const engine = new Engine();
    engine.setLoader(loader);
    const ctx = engine.createContext();
    ctx.eval_module('import { v } from "k"; globalThis.out = v;');
    expect(ctx.eval("globalThis.out")).toBe("from-loader");
  });
});

describe("napi: Engine (host environment API)", () => {
  test("create_array / createArray allocates a JS array of requested length", () => {
    const engine = new Engine();
    const arr1 = engine.create_array(3) as unknown[];
    expect(Array.isArray(arr1)).toBe(true);
    expect(arr1.length).toBe(3);

    const arr2 = engine.createArray(5) as unknown[];
    expect(Array.isArray(arr2)).toBe(true);
    expect(arr2.length).toBe(5);
  });

  test("createString / createInt32 / createDouble build JS values", () => {
    const engine = new Engine();
    expect(engine.createString("hello")).toBe("hello");
    expect(engine.createInt32(42)).toBe(42);
    expect(engine.createDouble(1.5)).toBe(1.5);
  });

  test("has_global / hasGlobal reads the host global object", () => {
    const engine = new Engine();
    expect(engine.has_global("Math")).toBe(true);
    expect(engine.hasGlobal("Math")).toBe(true);
    expect(engine.hasGlobal("definitelyNotAGlobal")).toBe(false);
  });

  test("get_napi_version / getNapiVersion reports host N-API version", () => {
    const engine = new Engine();
    expect(engine.get_napi_version()).toBeGreaterThan(0);
    expect(engine.getNapiVersion()).toBeGreaterThan(0);
  });

  test("make_object / makeObject builds a JS object via host context", () => {
    const engine = new Engine();
    const obj = engine.makeObject() as { ok: boolean; engine: string };
    expect(obj.ok).toBe(true);
    expect(obj.engine).toBe("napi");
  });

  test("set_instance_data / setInstanceData persists host-level state", () => {
    const engine = new Engine();
    engine.setInstanceData("vm-state-camel");
    expect(engine.getInstanceData()).toBe("vm-state-camel");
  });

  test("is_disposed / isDisposed is false while addon is loaded", () => {
    const engine = new Engine();
    expect(engine.is_disposed()).toBe(false);
    expect(engine.isDisposed()).toBe(false);
  });

  test("each instance captures its own context handle", () => {
    const a = new Engine();
    const b = new Engine();
    expect(a.isDisposed()).toBe(false);
    expect(b.isDisposed()).toBe(false);
  });
});
