import { expect, test } from "vite-plus/test";
import { Context, Engine } from "./main.ts";

test("Context prototype has expected methods", () => {
  expect(typeof Context.prototype.asRaw).toBe("function");
  expect(typeof Context.prototype.getEnginePtr).toBe("function");
  expect(typeof Context.prototype.Engine).toBe("function");
  expect(typeof Context.prototype.with).toBe("function");
});

test("Context has 5 static constructor methods", () => {
  expect(typeof Context.fromRaw).toBe("function");
  expect(typeof Context.base).toBe("function");
  expect(typeof Context.custom).toBe("function");
  expect(typeof Context.full).toBe("function");
  expect(typeof Context.builder).toBe("function");
});

test("Context.full creates a functional context", () => {
  const ctx = Context.full();
  expect(ctx).toBeInstanceOf(Context);
  expect(ctx.eval("2 + 3")).toBe(5);
});

test("Context.base creates a base context", () => {
  const ctx = Context.base();
  expect(ctx).toBeInstanceOf(Context);
  expect(typeof ctx.asRaw()).toBe("bigint");
});

test("Context.custom creates a custom context", () => {
  const ctx = Context.custom(["All"]);
  expect(ctx).toBeInstanceOf(Context);
  expect(ctx.eval("'hello' + ' world'")).toBe("hello world");
});

test("Context.builder creates a context", () => {
  const ctx = Context.builder();
  expect(ctx).toBeInstanceOf(Context);
  expect(typeof ctx.asRaw()).toBe("bigint");
});

test("Context constructors accept an Engine instance", () => {
  const engine = new Engine();
  const ctxFull = Context.full(engine);
  expect(ctxFull.eval("1 + 1")).toBe(2);

  const ctxBase = Context.base(engine);
  expect(ctxBase).toBeInstanceOf(Context);

  const ctxCustom = Context.custom(["All"], engine);
  expect(ctxCustom.eval("3 * 3")).toBe(9);

  const ctxBuilder = Context.builder(engine);
  expect(ctxBuilder).toBeInstanceOf(Context);
});

test("Context.prototype.asRaw and getEnginePtr return bigints", () => {
  const ctx = new Context();
  const ctxPtr = ctx.asRaw();
  const enginePtr = ctx.getEnginePtr();

  expect(typeof ctxPtr).toBe("bigint");
  expect(ctxPtr).toBeGreaterThan(0n);

  expect(typeof enginePtr).toBe("bigint");
  expect(enginePtr).toBeGreaterThan(0n);
});

test("Context.prototype.Engine returns the associated Engine", () => {
  const ctx = new Context();
  const engine = ctx.Engine();
  expect(engine).toBeInstanceOf(Engine);
});

test("Context.fromRaw creates a context from pointer and Engine", () => {
  const engine = new Engine();
  const originalCtx = Context.full(engine);
  const ptr = originalCtx.asRaw();

  const restoredCtx = Context.fromRaw(ptr, engine);
  expect(restoredCtx).toBeInstanceOf(Context);
  expect(restoredCtx.eval("123 + 456")).toBe(579);
});

test("Context.prototype.with executes callback within context", () => {
  const ctx = new Context();
  let executed = false;
  ctx.with((c: any) => {
    executed = true;
    expect(c.eval("1 + 1")).toBe(2);
  });
  expect(executed).toBe(true);
});
