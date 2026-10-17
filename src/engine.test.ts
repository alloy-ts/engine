import { expect, test } from "vite-plus/test";
import { Context, Engine } from "./main.ts";

test("Engine default constructor and factory constructors", () => {
  const defaultEngine = new Engine();
  expect(defaultEngine).toBeInstanceOf(Engine);

  const quickjsEngine = Engine.quickjs();
  expect(quickjsEngine).toBeInstanceOf(Engine);

  const boaEngine = Engine.boa();
  expect(boaEngine).toBeInstanceOf(Engine);
});

test("Engine creates Context instances", () => {
  const engine = new Engine();
  const ctx = engine.createContext();
  expect(ctx).toBeInstanceOf(Context);
  expect(ctx.eval("10 + 20")).toBe(30);
});

test("Engine creates QuickjsContext instances", () => {
  const engine = new Engine();
  const qjsCtx = engine.createQuickjsContext();
  expect(qjsCtx.eval("'quickjs' + ' engine'")).toBe("quickjs engine");
});

test("Engine memory limits and usage statistics", () => {
  const engine = new Engine();
  expect(() => engine.setMemoryLimit(1024 * 1024)).not.toThrow();
  expect(() => engine.setMaxStackSize(512 * 1024)).not.toThrow();
  expect(() => engine.setGcThreshold(256 * 1024)).not.toThrow();
  expect(() => engine.runGc()).not.toThrow();

  const usage = engine.memoryUsage();
  expect(usage).toBeDefined();
  expect(typeof usage).toBe("object");
});

test("Engine host NAPI environment functions", () => {
  const engine = new Engine();
  expect(engine.isDisposed()).toBe(false);
  expect(typeof engine.getNapiVersion()).toBe("number");
  expect(Array.isArray(engine.createArray(5))).toBe(true);
  expect(engine.createString("hello")).toBe("hello");
  expect(engine.createInt32(42)).toBe(42);
  expect(engine.createDouble(3.14159)).toBe(3.14159);

  engine.setInstanceData("test_instance_data");
  expect(engine.getInstanceData()).toBe("test_instance_data");
});

test("Engine weak handle", () => {
  const engine = new Engine();
  const weakRef = engine.weak();
  expect(weakRef).toBeDefined();
  const reacquired = weakRef.tryRef();
  expect(reacquired).not.toBeNull();
});
