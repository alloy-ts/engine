import { expect, test } from "vite-plus/test";
import { Context, Engine } from "./main.ts";

test("Context has all 5 static constructor methods", () => {
  expect(typeof Context.fromRaw).toBe("function");
  expect(typeof Context.base).toBe("function");
  expect(typeof Context.custom).toBe("function");
  expect(typeof Context.full).toBe("function");
  expect(typeof Context.builder).toBe("function");
});

test("Context.prototype has asRaw, getEnginePtr, Engine, and with", () => {
  expect(typeof Context.prototype.asRaw).toBe("function");
  expect(typeof Context.prototype.getEnginePtr).toBe("function");
  expect(typeof Context.prototype.Engine).toBe("function");
  expect(typeof Context.prototype.with).toBe("function");
});

test("Context.full creates a context backed by Engine", () => {
  const engine = new Engine();
  const ctx = Context.full(engine);
  expect(ctx.eval("2 + 3")).toBe(5);
});

test("Context.base creates a base context", () => {
  const engine = new Engine();
  const ctx = Context.base(engine);
  expect(ctx).toBeDefined();
});

test("Context.custom creates a custom context", () => {
  const engine = new Engine();
  const ctx = Context.custom(engine);
  expect(ctx).toBeDefined();
});

test("Context.builder creates a context builder and builds a context", () => {
  const engine = new Engine();
  const builder = Context.builder();
  expect(builder).toBeDefined();
  expect(typeof builder.build).toBe("function");
  const ctx = builder.build(engine);
  expect(ctx.eval("10 + 20")).toBe(30);
});

test("Context instance methods asRaw, getEnginePtr, Engine, with", () => {
  const engine = new Engine();
  const ctx = Context.full(engine);

  const rawCtx = ctx.asRaw();
  expect(rawCtx).toBeDefined();

  const rawEngine = ctx.getEnginePtr();
  expect(rawEngine).toBeDefined();

  const associatedEngine = ctx.Engine();
  expect(associatedEngine).toBeDefined();

  // Test fromRaw static constructor using rawCtx
  const ctxFromRaw = Context.fromRaw(rawCtx, engine);
  expect(ctxFromRaw.eval("1 + 1")).toBe(2);

  // Test prototype.with method
  const result = ctx.with((c: any) => {
    return c.eval("'hello from with'");
  });
  expect(result).toBe("hello from with");
});
