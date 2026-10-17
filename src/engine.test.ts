import { expect, test } from "vite-plus/test";
import { Engine, Context, QjsContext } from "./main.ts";

test("Engine default constructor and methods", () => {
  const engine = new Engine();
  expect(engine).toBeInstanceOf(Engine);
  expect(engine.isDisposed()).toBe(false);
});

test("Engine.backend constants", () => {
  const backend = typeof Engine.backend === "function" ? Engine.backend() : Engine.backend;
  expect(backend).toBeDefined();
  expect(backend.boa).toBe("boa");
  expect(backend.quickJs).toBe("quickjs");
});

test("Engine.quickjs factory constructor", () => {
  const engine = Engine.quickjs();
  expect(engine).toBeInstanceOf(Engine);
});

test("Engine.createContext creates a Boa-backed Context", () => {
  const engine = new Engine();
  const ctx = engine.createContext();
  expect(ctx).toBeInstanceOf(Context);
  expect(ctx.eval("10 + 20")).toBe(30);
});

test("Engine.createQuickjsContext creates a QuickJS-backed QjsContext", () => {
  const engine = new Engine();
  const qjsCtx = engine.createQuickjsContext();
  expect(qjsCtx).toBeDefined();
  expect(qjsCtx.eval("100 + 200")).toBe(300);
});

test("Engine memory and GC operations", () => {
  const engine = new Engine();
  expect(() => engine.setMemoryLimit(1024 * 1024)).not.toThrow();
  expect(() => engine.setMaxStackSize(256 * 1024)).not.toThrow();
  expect(() => engine.setGcThreshold(256 * 1024)).not.toThrow();
  expect(() => engine.runGc()).not.toThrow();

  const mem = engine.memoryUsage();
  expect(typeof mem).toBe("object");
});

test("Engine host N-API operations", () => {
  const engine = new Engine();
  expect(Array.isArray(engine.createArray(3))).toBe(true);
  expect(engine.createString("test")).toBe("test");
  expect(engine.createInt32(42)).toBe(42);
  expect(engine.createDouble(3.14)).toBe(3.14);
  expect(typeof engine.getNapiVersion()).toBe("number");
  expect(engine.makeObject()).toEqual({ ok: true, engine: "napi" });
});
