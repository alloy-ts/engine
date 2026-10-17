import { expect, test } from "vite-plus/test";
import { Engine } from "./main.ts";

test("Engine default instantiation (boa)", () => {
  const engine = new Engine();
  expect(engine.backend).toBe("boa");
  expect(engine.eval("1 + 1")).toBe(2);
});

test("Engine.boa() and Engine.quickjs() factory methods", () => {
  const boa = Engine.boa();
  expect(boa.backend).toBe("boa");
  expect(boa.eval("2 * 3")).toBe(6);

  const qjs = Engine.quickjs();
  expect(qjs.backend).toBe("quickjs");
  expect(qjs.eval("10 - 4")).toBe(6);
});

test("Engine.createContext() creates an isolated context", () => {
  const engine = new Engine();
  const ctx = engine.createContext();
  expect(ctx.eval("console.log(3 + 3.33333); 5")).toBe(5);
  ctx.dispose();
});

test("Engine GC and memory usage", () => {
  const engine = new Engine();
  expect(() => engine.runGc()).not.toThrow();
  const usage = engine.memoryUsage();
  expect(usage).toHaveProperty("backend");
  expect(usage).toHaveProperty("engine");
});

test("Engine runtime limits and optimizer options", () => {
  const engine = new Engine();
  const limits = engine.runtimeLimits();
  expect(limits).toHaveProperty("stackSize");
  engine.setRuntimeLimits({ stackSize: 256 * 1024 });

  const opts = engine.optimizerOptions();
  expect(opts).toHaveProperty("constantFolding");
  engine.setOptimizerOptions({ constantFolding: true });
});

test("Engine weak handle", () => {
  const engine = new Engine();
  const weak = engine.weak();
  expect(weak.isAlive()).toBe(true);
});
