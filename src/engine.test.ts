import { expect, test } from "vite-plus/test";
import { Engine } from "./main.ts";

test("new Engine() defaults to boa backend", () => {
  const engine = new Engine();
  expect(engine).toBeDefined();
  expect(engine.isDisposed()).toBe(false);
  expect(engine.version()).toBe("0.22.0");
  expect(Array.isArray(engine.features())).toBe(true);
});

test("Engine.backend constants are defined", () => {
  expect(Engine.backend).toBeDefined();
  expect(Engine.backend.Boa).toBe("boa");
  expect(Engine.backend.QuickJs).toBe("quickjs");
});

test("Engine.createContext() returns a usable Context", () => {
  const engine = new Engine();
  const ctx = engine.createContext();
  expect(ctx.eval("2 + 3")).toBe(5);
  ctx.dispose();
});

test("Engine.parseScript() returns a parsed Script", () => {
  const engine = new Engine();
  const script = engine.parseScript("10 * 4");
  expect(script.evaluate()).toBe(40);
});

test("Engine host helper methods", () => {
  const engine = new Engine();
  expect(engine.createString("hello")).toBe("hello");
  expect(engine.createInt32(42)).toBe(42);
  expect(engine.createDouble(3.14)).toBe(3.14);
  expect(Array.isArray(engine.createArray(3))).toBe(true);
  expect(typeof engine.getNapiVersion()).toBe("number");
  expect(engine.makeObject()).toEqual({ ok: true, engine: "napi" });
  expect(engine.hasGlobal("Object")).toBe(true);
});

test("Engine instance data persistence", () => {
  const engine = new Engine();
  engine.setInstanceData("test-data");
  expect(engine.getInstanceData()).toBe("test-data");
});

test("Engine.quickjs() constructor sentinel handling", () => {
  // QuickJS feature is not compiled in by default, so it should throw a clear error.
  expect(() => Engine.quickjs()).toThrow(/QuickJS backend not compiled in/);
  expect(() => new Engine(Engine.backend.QuickJs)).toThrow(
    /QuickJS backend not compiled in/,
  );
  expect(() => new Engine("unknown" as any)).toThrow(/unknown engine backend/);
});
