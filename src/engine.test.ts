import { expect, test } from "vite-plus/test";
import { Engine } from "./main.ts";

test("new Engine() defaults to boa backend", () => {
  const engine = new Engine();
  expect(engine).toBeDefined();
  expect(engine.isDisposed()).toBe(false);
});

test("Engine.backend has Boa and QuickJs sentinels", () => {
  expect(Engine.backend).toEqual({
    Boa: "boa",
    QuickJs: "quickjs",
  });
});

test("new Engine(Engine.backend.Boa) creates a boa engine", () => {
  const engine = new Engine(Engine.backend.Boa);
  expect(engine).toBeDefined();
});

test("new Engine with invalid backend throws error", () => {
  expect(() => new (Engine as any)("invalid_backend")).toThrow(
    /unknown engine backend: invalid_backend/
  );
});

test("Engine.quickjs() and Engine(Engine.backend.QuickJs) construct quickjs backend or throw cleanly if not compiled", () => {
  try {
    const e1 = new Engine(Engine.backend.QuickJs);
    expect(e1).toBeDefined();
  } catch (err: any) {
    expect(err.message).toMatch(/QuickJS backend not compiled in/);
  }

  try {
    const e2 = Engine.quickjs();
    expect(e2).toBeDefined();
  } catch (err: any) {
    expect(err.message).toMatch(/QuickJS backend not compiled in/);
  }
});

test("engine.createContext() creates an execution context", () => {
  const engine = new Engine();
  const ctx = engine.createContext();
  expect(ctx.eval("20 + 22")).toBe(42);
  ctx.dispose();
});

test("engine.parseScript() parses code into a reusable Script", () => {
  const engine = new Engine();
  const script = engine.parseScript("5 * 5");
  expect(script.evaluate()).toBe(25);
  expect(script.source).toBe("5 * 5");
});

test("engine.version() returns boa_engine crate version", () => {
  const engine = new Engine();
  expect(engine.version()).toBe("0.22.0");
});

test("engine.features() returns array of feature flags", () => {
  const engine = new Engine();
  const features = engine.features();
  expect(Array.isArray(features)).toBe(true);
});

test("engine.getNapiVersion() returns host N-API version", () => {
  const engine = new Engine();
  expect(typeof engine.getNapiVersion()).toBe("number");
  expect(engine.getNapiVersion()).toBeGreaterThan(0);
});

test("host object creation helpers", () => {
  const engine = new Engine();
  expect(engine.createString("hello")).toBe("hello");
  expect(engine.createInt32(123)).toBe(123);
  expect(engine.createDouble(3.14159)).toBe(3.14159);

  const arr = engine.createArray(5);
  expect(Array.isArray(arr)).toBe(true);
  expect(arr.length).toBe(5);

  const obj = engine.makeObject();
  expect(obj).toEqual({ ok: true, engine: "napi" });
});

test("engine.hasGlobal() checks host global property", () => {
  const engine = new Engine();
  expect(engine.hasGlobal("console")).toBe(true);
  expect(engine.hasGlobal("nonExistentGlobalXYZ123")).toBe(false);
});

test("instance data persistence on engine", () => {
  const engine = new Engine();
  engine.setInstanceData("my_instance_data");
  expect(engine.getInstanceData()).toBe("my_instance_data");
});
