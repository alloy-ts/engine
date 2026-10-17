import { describe, expect, test } from "vite-plus/test";
import { Context, Engine } from "./main.ts";

describe("napi: Context static constructors", () => {
  test("Context.full() constructs a context", () => {
    const ctx = Context.full();
    expect(ctx).toBeInstanceOf(Context);
    expect(ctx.eval("1 + 1")).toBe(2);

    const engine = new Engine();
    const ctxWithEngine = Context.full(engine);
    expect(ctxWithEngine).toBeInstanceOf(Context);
    expect(ctxWithEngine.eval("2 + 2")).toBe(4);
  });

  test("Context.base() constructs a context", () => {
    const ctx = Context.base();
    expect(ctx).toBeInstanceOf(Context);

    const engine = new Engine();
    const ctxWithEngine = Context.base(engine);
    expect(ctxWithEngine).toBeInstanceOf(Context);
  });

  test("Context.custom() constructs a context", () => {
    const ctx = Context.custom();
    expect(ctx).toBeInstanceOf(Context);

    const engine = new Engine();
    const ctxWithEngine = Context.custom(engine, ["All"]);
    expect(ctxWithEngine).toBeInstanceOf(Context);
  });

  test("Context.builder() constructs a context", () => {
    const ctx = Context.builder();
    expect(ctx).toBeInstanceOf(Context);

    const engine = new Engine();
    const ctxWithEngine = Context.builder(engine);
    expect(ctxWithEngine).toBeInstanceOf(Context);
  });

  test("Context.fromRaw() constructs a context from raw pointers", () => {
    const engine = new Engine();
    const origCtx = engine.create_context();
    const rawPtr = origCtx.asRaw();
    expect(typeof rawPtr).toBe("bigint");

    const reimportedCtx = Context.fromRaw(rawPtr, engine);
    expect(reimportedCtx).toBeInstanceOf(Context);
    expect(reimportedCtx.eval("3 * 3")).toBe(9);
  });
});

describe("napi: Context prototype instance methods", () => {
  test("Context.prototype contains required methods", () => {
    expect(typeof Context.prototype.asRaw).toBe("function");
    expect(typeof Context.prototype.getEnginePtr).toBe("function");
    expect(typeof Context.prototype.Engine).toBe("function");
    expect(typeof Context.prototype.with).toBe("function");
  });

  test("asRaw returns bigint pointer", () => {
    const ctx = new Context();
    const raw = ctx.asRaw();
    expect(typeof raw).toBe("bigint");
    expect(raw).toBeGreaterThan(0n);
  });

  test("getEnginePtr returns bigint pointer", () => {
    const ctx = new Context();
    const enginePtr = ctx.getEnginePtr();
    expect(typeof enginePtr).toBe("bigint");
    expect(enginePtr).toBeGreaterThan(0n);
  });

  test("Engine method returns Engine instance", () => {
    const ctx = new Context();
    const eng = ctx.Engine();
    expect(eng).toBeDefined();
    expect(typeof eng.create_context).toBe("function");

    const eng2 = ctx.engine();
    expect(eng2).toBeDefined();
  });

  test("with method invokes callback with Ctx handle", () => {
    const ctx = new Context();
    let called = false;
    ctx.with((c: any) => {
      called = true;
      expect(c).toBeDefined();
      expect(c.eval("10 + 20")).toBe(30);
    });
    expect(called).toBe(true);
  });
});
