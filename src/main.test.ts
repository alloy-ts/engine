import { expect, test } from "vite-plus/test";
import { main, Context, Engine } from "./main.ts";

test("main returns Hello, world!", () => {
  expect(main()).toBe("Hello, world!");
});

test("Context.full() creates a full context", () => {
  const ctx = Context.full();
  expect(ctx).toBeInstanceOf(Context);
  expect(ctx.eval("1 + 2")).toBe(3);
});

test("Context.base() creates a base context with valid raw pointer", () => {
  const ctx = Context.base();
  expect(ctx).toBeInstanceOf(Context);
  expect(ctx.asRaw()).toBeGreaterThan(0n);
});

test("Context.custom() creates a custom context", () => {
  const ctx = Context.custom(undefined, ["Eval"]);
  expect(ctx).toBeInstanceOf(Context);
  expect(ctx.eval("2 * 3")).toBe(6);
});

test("Context.builder() returns a builder and builds a context", () => {
  const builder = Context.builder();
  const ctx = builder.build();
  expect(ctx).toBeInstanceOf(Context);
  expect(ctx.eval("4 + 4")).toBe(8);
});

test("Context static constructors work with explicit Engine", () => {
  const engine = new Engine();
  const baseCtx = Context.base(engine);
  const customCtx = Context.custom(engine, ["Eval"]);
  const fullCtx = Context.full(engine);
  const builderCtx = Context.builder(engine).build(engine);

  expect(baseCtx.asRaw()).toBeGreaterThan(0n);
  expect(customCtx.eval("10 + 10")).toBe(20);
  expect(fullCtx.eval("10 + 15")).toBe(25);
  expect(builderCtx.eval("10 + 20")).toBe(30);
});

test("Context.prototype.asRaw() returns a non-zero bigint pointer", () => {
  const ctx = Context.full();
  const rawPtr = ctx.asRaw();
  expect(typeof rawPtr).toBe("bigint");
  expect(rawPtr).toBeGreaterThan(0n);
});

test("Context.prototype.getEnginePtr() returns a non-zero bigint pointer", () => {
  const ctx = Context.full();
  const enginePtr = ctx.getEnginePtr();
  expect(typeof enginePtr).toBe("bigint");
  expect(enginePtr).toBeGreaterThan(0n);
});

test("Context.prototype.Engine() and engine() return the Engine instance", () => {
  const ctx = Context.full();
  const eng1 = ctx.Engine();
  const eng2 = ctx.engine();
  expect(eng1).toBeInstanceOf(Engine);
  expect(eng2).toBeInstanceOf(Engine);
  expect(eng1.isJobPending()).toBe(false);
});

test("Context.prototype.with() passes Ctx to the callback", () => {
  const ctx = Context.full();
  const result = ctx.with((c) => {
    return c.eval("'hello from with'");
  });
  expect(result).toBe("hello from with");
});

test("Context.fromRaw() constructs a context from a raw pointer and Engine", () => {
  const engine = new Engine();
  const originalCtx = Context.full(engine);
  const ptr = originalCtx.asRaw();

  const restoredCtx = Context.fromRaw(ptr, engine);
  expect(restoredCtx).toBeInstanceOf(Context);
  expect(restoredCtx.eval("100 + 200")).toBe(300);
});
