import { expect, test } from "vite-plus/test";
import { Engine, Script } from "./main.ts";

test("Engine default constructor creates a boa-backed engine", () => {
  const engine = new Engine();
  expect(engine).toBeDefined();
  expect(engine.isDisposed()).toBe(false);
  expect(engine.version()).toBe("0.22.0");
  expect(Array.isArray(engine.features())).toBe(true);
});

test("Engine creates isolated Contexts", () => {
  const engine = new Engine();
  const ctx1 = engine.createContext();
  const ctx2 = engine.createContext();

  expect(ctx1.eval("1 + 1")).toBe(2);
  expect(ctx2.eval("2 * 3")).toBe(6);

  ctx1.registerGlobalProperty("a", 100, 7);
  expect(ctx1.eval("a")).toBe(100);
  expect(() => ctx2.eval("a")).toThrow();

  ctx1.dispose();
  ctx2.dispose();
});

test("Engine parses reusable Script", () => {
  const engine = new Engine();
  const script = engine.parseScript("10 + 20");
  expect(script).toBeInstanceOf(Script);
  expect(script.evaluate()).toBe(30);
});

test("Engine host helper functions", () => {
  const engine = new Engine();
  expect(engine.createString("hello")).toBe("hello");
  expect(engine.createInt32(42)).toBe(42);
  expect(engine.createDouble(3.14159)).toBe(3.14159);
  expect(engine.createArray(3)).toHaveLength(3);
  expect(typeof engine.getNapiVersion()).toBe("number");
  expect(engine.hasGlobal("console")).toBe(true);
});

test("Engine backend selection sentinel", () => {
  expect(Engine.backend).toEqual({
    Boa: "boa",
    QuickJs: "quickjs",
  });
});
