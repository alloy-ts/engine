import { expect, test } from "vite-plus/test";
import { Engine, Loader } from "./main.ts";

test("Engine constructor and context creation", () => {
  const engine = new Engine();
  expect(engine.isDisposed()).toBe(false);

  const ctx = engine.createContext();
  expect(ctx.eval("1 + 1")).toBe(2);
});

test("Engine Loader registry", () => {
  const loader = new Loader();
  loader.add("calc", "export const multiply = (a, b) => a * b;");
  expect(loader.has("calc")).toBe(true);
  expect(loader.keys()).toContain("calc");

  const engine = new Engine(loader);
  expect(engine.isDisposed()).toBe(false);

  expect(loader.remove("calc")).toBe(true);
  expect(loader.has("calc")).toBe(false);
});

test("Engine QuickJS context creation", () => {
  const engine = new Engine();
  const qjsCtx = engine.createQuickjsContext();
  expect(qjsCtx.eval("20 + 30")).toBe(50);
});

test("Engine memory limits and tuning", () => {
  const engine = new Engine();
  expect(() => engine.setMemoryLimit(10 * 1024 * 1024)).not.toThrow();
  expect(() => engine.setMaxStackSize(512 * 1024)).not.toThrow();
  expect(() => engine.setGcThreshold(1024 * 1024)).not.toThrow();
  expect(() => engine.setInfo("test-engine")).not.toThrow();
});

test("Engine garbage collection and memory usage", () => {
  const engine = new Engine();
  expect(() => engine.runGc()).not.toThrow();

  const usage = engine.memoryUsage() as Record<string, number>;
  expect(usage).toBeDefined();
  expect(typeof usage.memory_used_size).toBe("number");
});

test("Engine job pending and job execution", () => {
  const engine = new Engine();
  expect(typeof engine.isJobPending()).toBe("boolean");
  expect(typeof engine.executePendingJob()).toBe("boolean");
});

test("Engine weak handles", () => {
  const engine = new Engine();
  const weak = engine.weak();
  const ref = weak.tryRef();
  expect(ref).not.toBeNull();
  expect(ref?.isDisposed()).toBe(false);
});

test("Engine host N-API utilities", () => {
  const engine = new Engine();

  const arr = engine.createArray(5);
  expect(Array.isArray(arr)).toBe(true);
  expect(arr.length).toBe(5);

  const str = engine.createString("alloy");
  expect(str).toBe("alloy");

  const i32 = engine.createInt32(100);
  expect(i32).toBe(100);

  const dbl = engine.createDouble(3.14159);
  expect(dbl).toBe(3.14159);

  expect(engine.hasGlobal("Math")).toBe(true);
  expect(engine.getNapiVersion()).toBeGreaterThan(0);

  const obj = engine.makeObject() as Record<string, unknown>;
  expect(obj.ok).toBe(true);
  expect(obj.engine).toBe("napi");

  engine.setInstanceData("my-engine-session");
  expect(engine.getInstanceData()).toBe("my-engine-session");
});
