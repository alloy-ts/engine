import { expect, test } from "vite-plus/test";
import { Context, ContextBuilder, Engine } from "./main.ts";

test("Context prototype has required methods", () => {
  expect(typeof Context.prototype.asRaw).toBe("function");
  expect(typeof Context.prototype.getEnginePtr).toBe("function");
  expect(typeof Context.prototype.Engine).toBe("function");
  expect(typeof Context.prototype.with).toBe("function");
});

test("Context has required static constructors", () => {
  expect(typeof Context.fromRaw).toBe("function");
  expect(typeof Context.base).toBe("function");
  expect(typeof Context.custom).toBe("function");
  expect(typeof Context.full).toBe("function");
  expect(typeof Context.builder).toBe("function");
});

test("Context.full creates a working context", () => {
  const engine = new Engine();
  const ctx = Context.full(engine);
  expect(ctx).toBeInstanceOf(Context);
  expect(ctx.eval("1 + 2")).toBe(3);
});

test("Context.base creates a working base context", () => {
  const engine = new Engine();
  const ctx = Context.base(engine);
  expect(ctx).toBeInstanceOf(Context);
  expect(ctx.asRaw()).not.toBe(0n);
});

test("Context.custom creates a working custom context", () => {
  const engine = new Engine();
  const ctx = Context.custom(engine);
  expect(ctx).toBeInstanceOf(Context);
  expect(ctx.asRaw()).not.toBe(0n);
});

test("Context.builder returns a ContextBuilder and builds a context", () => {
  const builder = Context.builder();
  expect(builder).toBeInstanceOf(ContextBuilder);
  const engine = new Engine();
  const ctx = builder.build(engine);
  expect(ctx).toBeInstanceOf(Context);
  expect(ctx.asRaw()).not.toBe(0n);
});

test("Context instance methods asRaw, getEnginePtr, Engine, and with", () => {
  const engine = new Engine();
  const ctx = Context.full(engine);

  const rawPtr = ctx.asRaw();
  expect(typeof rawPtr).toBe("bigint");
  expect(rawPtr).not.toBe(0n);

  const enginePtr = ctx.getEnginePtr();
  expect(typeof enginePtr).toBe("bigint");
  expect(enginePtr).not.toBe(0n);

  const ctxEngine = ctx.Engine();
  expect(ctxEngine).toBeInstanceOf(Engine);

  const res = ctx.with(() => 42);
  expect(res).toBe(42);

  // Test Context.fromRaw
  const ctxFromRaw = Context.fromRaw(rawPtr, engine);
  expect(ctxFromRaw).toBeInstanceOf(Context);
  expect(ctxFromRaw.eval("100 + 200")).toBe(300);
});
