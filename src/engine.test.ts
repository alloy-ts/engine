import { expect, test } from "vitest";
import { Engine } from "./main.ts";

test("Engine default constructor creates a boa engine facade", () => {
  const engine = new Engine();
  expect(engine).toBeDefined();
  expect(engine.version()).toBe("0.22.0");
  expect(engine.isDisposed()).toBe(false);
});

test("Engine.backend selectors and quickjs constructor", () => {
  expect(Engine.backend).toBeDefined();
  expect(Engine.backend.Boa).toBe("boa");
  expect(Engine.backend.QuickJs).toBe("quickjs");

  // Attempting quickjs backend throws if feature not compiled in
  expect(() => Engine.quickjs()).toThrow(/QuickJS backend not compiled in/);
});

test("Engine.createContext creates an isolated Context", () => {
  const engine = new Engine();
  const ctx = engine.createContext();
  expect(ctx).toBeDefined();
  expect(ctx.eval("2 + 3")).toBe(5);
  ctx.dispose();
});

test("Engine.parseScript parses executable Script", () => {
  const engine = new Engine();
  const script = engine.parseScript("console.log(3 + 3.33333); 10 * 10");
  expect(script).toBeDefined();
  expect(script.evaluate()).toBe(100);
});

test("Engine host N-API helpers (createArray, createString, createInt32, makeObject)", () => {
  const engine = new Engine();
  const arr = engine.createArray(3);
  expect(Array.isArray(arr)).toBe(true);
  expect(arr.length).toBe(3);

  const str = engine.createString("hello engine");
  expect(str).toBe("hello engine");

  const num = engine.createInt32(42);
  expect(num).toBe(42);

  const dbl = engine.createDouble(3.14159);
  expect(dbl).toBe(3.14159);

  const obj = engine.makeObject();
  expect(obj).toEqual({ ok: true, engine: "napi" });
});

test("Engine instance data get/set", () => {
  const engine = new Engine();
  expect(engine.getInstanceData()).toBe(null);
  engine.setInstanceData("my-custom-data");
  expect(engine.getInstanceData()).toBe("my-custom-data");
});
