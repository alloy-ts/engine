import { describe, expect, it } from "vite-plus/test";
import { Context, Engine, JsCtx } from "./main.ts";

describe("Context NAPI bindings", () => {
  it("has the 5 static constructor methods on Context class", () => {
    expect(typeof Context.fromRaw).toBe("function");
    expect(typeof Context.base).toBe("function");
    expect(typeof Context.custom).toBe("function");
    expect(typeof Context.full).toBe("function");
    expect(typeof Context.builder).toBe("function");
  });

  it("has the 4 prototype methods on Context.prototype", () => {
    expect(typeof Context.prototype.asRaw).toBe("function");
    expect(typeof Context.prototype.getEnginePtr).toBe("function");
    expect(typeof Context.prototype.Engine).toBe("function");
    expect(typeof Context.prototype.with).toBe("function");
  });

  it("Context.full creates a full context and evaluates JS", () => {
    const ctx = Context.full();
    expect(ctx).toBeInstanceOf(Context);
    expect(ctx.eval("2 + 3")).toBe(5);
  });

  it("Context.full accepts an Engine instance", () => {
    const engine = new Engine();
    const ctx = Context.full(engine);
    expect(ctx.eval("'hello ' + 'world'")).toBe("hello world");
  });

  it("Context.base creates a base context", () => {
    const engine = new Engine();
    const ctx = Context.base(engine);
    expect(ctx).toBeInstanceOf(Context);
    expect(typeof ctx.asRaw()).toBe("bigint");
    expect(ctx.asRaw()).not.toBe(0n);
  });

  it("Context.custom creates a custom context with intrinsics", () => {
    const engine = new Engine();
    const ctx = Context.custom(engine, ["Eval"]);
    expect(ctx).toBeInstanceOf(Context);
    expect(ctx.eval("40 + 2")).toBe(42);
  });

  it("Context.builder constructs context using builder pattern with multiple combined intrinsics", () => {
    const engine = new Engine();
    const builder = Context.builder();
    builder.withEval().withJson().withRegexp();
    const ctx = builder.build(engine);
    expect(ctx).toBeInstanceOf(Context);
    expect(ctx.eval("JSON.stringify(/abc/.test('abc'))")).toBe("true");
  });

  it("Context.prototype.asRaw returns a valid bigint pointer", () => {
    const ctx = new Context();
    const rawPtr = ctx.asRaw();
    expect(typeof rawPtr).toBe("bigint");
    expect(rawPtr).not.toBe(0n);
  });

  it("Context.prototype.getEnginePtr returns a valid bigint pointer", () => {
    const ctx = new Context();
    const enginePtr = ctx.getEnginePtr();
    expect(typeof enginePtr).toBe("bigint");
    expect(enginePtr).not.toBe(0n);
  });

  it("Context.prototype.Engine returns the associated Engine instance", () => {
    const ctx = new Context();
    const engine = ctx.Engine();
    expect(engine).toBeDefined();
    expect(typeof engine.runGc).toBe("function");
  });

  it("Context.prototype.with executes callback within context scope", () => {
    const ctx = Context.full();
    let executed = false;
    ctx.with((jsCtx) => {
      executed = true;
      expect(jsCtx).toBeDefined();
      expect((jsCtx as JsCtx).eval("1 + 1")).toBe(2);
    });
    expect(executed).toBe(true);
  });

  it("Context.fromRaw restores a Context from raw pointer and Engine", () => {
    const engine = new Engine();
    const originalCtx = Context.full(engine);
    const rawPtr = originalCtx.asRaw();
    expect(typeof rawPtr).toBe("bigint");

    const restoredCtx = Context.fromRaw(rawPtr, engine);
    expect(restoredCtx).toBeInstanceOf(Context);
    expect(restoredCtx.eval("'from_raw'")).toBe("from_raw");
  });
});
