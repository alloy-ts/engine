import { expect, test } from "vite-plus/test";
import { Engine, Loader } from "./main.ts";

test("Engine.backend returns backend map", () => {
  const backend = Engine.backend();
  expect(backend).toEqual({ boa: "boa", quickjs: "quickjs" });
});

test("Engine constructors and factory methods", () => {
  const defaultEngine = new Engine();
  expect(defaultEngine.isDisposed()).toBe(false);

  const boaEngine = Engine.boa();
  expect(boaEngine.isDisposed()).toBe(false);

  const loader = new Loader();
  const qjsEngine = Engine.quickjs(loader);
  expect(qjsEngine.isDisposed()).toBe(false);
});

test("Engine createContext and createQuickjsContext", () => {
  const engine = new Engine();
  const boaCtx = engine.createContext();
  expect(boaCtx.eval("1 + 2")).toBe(3);

  const qjsCtx = engine.createQuickjsContext();
  expect(qjsCtx.eval("2 * 3")).toBe(6);
});

test("Engine memory limits and memory usage", () => {
  const engine = new Engine();
  engine.setMemoryLimit(1024 * 1024 * 16);
  engine.setMaxStackSize(256 * 1024);
  engine.setGcThreshold(64 * 1024);

  const mem = engine.memoryUsage();
  expect(mem).toHaveProperty("malloc_count");
  expect(mem).toHaveProperty("memory_used_size");
});

test("Engine GC and pending jobs", () => {
  const engine = new Engine();
  expect(engine.isJobPending()).toBe(false);
  expect(engine.executePendingJob()).toBe(false);
  engine.runGc();
});

test("Engine host NAPI value creation helpers", () => {
  const engine = new Engine();
  expect(engine.createArray(3)).toHaveLength(3);
  expect(engine.createString("hello")).toBe("hello");
  expect(engine.createInt32(42)).toBe(42);
  expect(engine.createDouble(3.14)).toBeCloseTo(3.14);

  const obj = engine.makeObject();
  expect(obj).toEqual({ ok: true, engine: "napi" });
});

test("Engine instance data persistence", () => {
  const engine = new Engine();
  expect(engine.getInstanceData()).toBeNull();
  engine.setInstanceData("my-host-data");
  expect(engine.getInstanceData()).toBe("my-host-data");
});

test("Engine weak handle tryRef", () => {
  const engine = new Engine();
  const weak = engine.weak();
  const promoted = weak.tryRef();
  expect(promoted).not.toBeNull();
  expect(promoted?.isDisposed()).toBe(false);
});
