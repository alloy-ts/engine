import { expect, test } from "vite-plus/test";
import { Context, Engine } from "./main.ts";

test("Context static factory constructors", () => {
  expect(typeof Context.full).toBe("function");
  expect(typeof Context.base).toBe("function");
  expect(typeof Context.custom).toBe("function");
  expect(typeof Context.builder).toBe("function");
  expect(typeof Context.fromRaw).toBe("function");
});

test("Context prototype methods", () => {
  expect(typeof Context.prototype.asRaw).toBe("function");
  expect(typeof Context.prototype.getEnginePtr).toBe("function");
  expect(typeof Context.prototype.Engine).toBe("function");
  expect(typeof Context.prototype.with).toBe("function");
});

test("Context static constructors and instance prototype methods invocation", () => {
  const ctx = Context.full();
  expect(ctx).toBeDefined();

  expect(typeof ctx.asRaw).toBe("function");
  expect(typeof ctx.getEnginePtr).toBe("function");
  expect(typeof ctx.Engine).toBe("function");
  expect(typeof ctx.with).toBe("function");

  const rawPtr = ctx.asRaw();
  expect(typeof rawPtr).toBe("bigint");

  const enginePtr = ctx.getEnginePtr();
  expect(typeof enginePtr).toBe("bigint");

  const engine = ctx.Engine();
  expect(engine).toBeDefined();

  if (typeof Context.fromRaw === "function") {
    const rawCtx = Context.fromRaw(rawPtr, engine);
    expect(rawCtx).toBeDefined();
  }

  const baseCtx = Context.base();
  expect(baseCtx).toBeDefined();

  const customCtx = Context.custom();
  expect(customCtx).toBeDefined();

  const builderCtx = Context.builder();
  expect(builderCtx).toBeDefined();

  let evaluatedValue: number | undefined;
  ctx.with((innerCtx: any) => {
    evaluatedValue = innerCtx.eval("2 + 3");
  });
  expect(evaluatedValue).toBe(5);
});
