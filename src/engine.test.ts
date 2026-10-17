import { expect, test } from "vite-plus/test";
import { Engine } from "./main.ts";

test("new Engine() defaults to boa backend", () => {
  const engine = new Engine();
  expect(engine).toBeDefined();
  expect(engine.isDisposed()).toBe(false);
});

test("Engine.backend constant exposes Boa and QuickJs sentinels", () => {
  expect(Engine.backend).toEqual({
    Boa: "boa",
    QuickJs: "quickjs",
  });
});

test("new Engine(Engine.backend.Boa) creates a boa engine", () => {
  const engine = new Engine(Engine.backend.Boa);
  expect(engine.version()).toBe("0.22.0");
});

test("engine.createContext() returns a working Context", () => {
  const engine = new Engine();
  const ctx = engine.createContext();
  expect(ctx.eval("21 * 2")).toBe(42);
  ctx.dispose();
});

test("engine.parseScript() parses code into a Script", () => {
  const engine = new Engine();
  const script = engine.parseScript("10 + 20");
  expect(script.evaluate()).toBe(30);
});

test("engine.version() returns boa version 0.22.0", () => {
  const engine = new Engine();
  expect(engine.version()).toBe("0.22.0");
});

test("engine.features() returns enabled features", () => {
  const engine = new Engine();
  expect(Array.isArray(engine.features())).toBe(true);
});

test("Host environment helpers: createArray, createString, createInt32, createDouble", () => {
  const engine = new Engine();
  const arr = engine.createArray(3);
  expect(Array.isArray(arr)).toBe(true);
  expect(arr.length).toBe(3);

  expect(engine.createString("hello")).toBe("hello");
  expect(engine.createInt32(100)).toBe(100);
  expect(engine.createDouble(3.14159)).toBe(3.14159);
});

test("Host environment helpers: hasGlobal and getNapiVersion", () => {
  const engine = new Engine();
  expect(typeof engine.hasGlobal("console")).toBe("boolean");
  expect(typeof engine.getNapiVersion()).toBe("number");
  expect(engine.getNapiVersion()).toBeGreaterThan(0);
});

test("Host environment helpers: makeObject", () => {
  const engine = new Engine();
  const obj = engine.makeObject();
  expect(obj).toEqual({ ok: true, engine: "napi" });
});

test("Host environment helpers: setInstanceData and getInstanceData", () => {
  const engine = new Engine();
  engine.setInstanceData("my-app-state");
  expect(engine.getInstanceData()).toBe("my-app-state");
});
