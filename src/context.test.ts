import { expect, test } from "vite-plus/test";
import { Context, Engine } from "./main.ts";

test("Context static constructor methods exist and construct instances", () => {
  expect(typeof Context.full).toBe("function");
  expect(typeof Context.base).toBe("function");
  expect(typeof Context.custom).toBe("function");
  expect(typeof Context.builder).toBe("function");
  expect(typeof Context.fromRaw).toBe("function");

  const engine = new Engine();

  const ctxFull = Context.full(engine);
  expect(ctxFull).toBeInstanceOf(Context);
  expect(ctxFull.eval("1 + 1")).toBe(2);

  const ctxBase = Context.base(engine);
  expect(ctxBase).toBeInstanceOf(Context);

  const ctxCustom = Context.custom(engine);
  expect(ctxCustom).toBeInstanceOf(Context);

  const ctxBuilder = Context.builder(engine);
  expect(ctxBuilder).toBeInstanceOf(Context);

  const ctxRaw = ctxFull.asRaw();
  const ctxFromRaw = Context.fromRaw(ctxRaw, engine);
  expect(ctxFromRaw).toBeInstanceOf(Context);
  expect(ctxFromRaw.eval("2 + 3")).toBe(5);
});

test("Context prototype methods exist and function correctly", () => {
  expect(typeof Context.prototype.asRaw).toBe("function");
  expect(typeof Context.prototype.getEnginePtr).toBe("function");
  expect(typeof Context.prototype.Engine).toBe("function");
  expect(typeof Context.prototype.with).toBe("function");

  const ctx = Context.full();

  const rawPtr = ctx.asRaw();
  expect(typeof rawPtr).toBe("bigint");
  expect(rawPtr).toBeGreaterThan(0n);

  const enginePtr = ctx.getEnginePtr();
  expect(typeof enginePtr).toBe("bigint");
  expect(enginePtr).toBeGreaterThan(0n);

  const engineInstance = ctx.Engine();
  expect(engineInstance).toBeDefined();
  expect(typeof engineInstance.createArray).toBe("function");

  let withExecuted = false;
  ctx.with((innerCtx: any) => {
    withExecuted = true;
    expect(innerCtx.eval("10 + 20")).toBe(30);
  });
  expect(withExecuted).toBe(true);
});
