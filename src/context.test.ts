import { describe, expect, test } from "vite-plus/test";
import { Context, Engine } from "./main.ts";

describe("napi: Context static constructor methods & prototype methods", () => {
  test("Context has expected static constructor methods", () => {
    expect(typeof Context.fromRaw).toBe("function");
    expect(typeof Context.base).toBe("function");
    expect(typeof Context.custom).toBe("function");
    expect(typeof Context.full).toBe("function");
    expect(typeof Context.builder).toBe("function");
  });

  test("Context.base() creates a valid Context instance", () => {
    const ctx = Context.base();
    expect(ctx).toBeInstanceOf(Context);
    expect(ctx.asRaw()).not.toBe(0n);
  });

  test("Context.custom() creates a valid Context instance", () => {
    const ctx = Context.custom(["All"]);
    expect(ctx).toBeInstanceOf(Context);
    expect(ctx.eval("2 + 3")).toBe(5);

    const minimalCtx = Context.custom();
    expect(minimalCtx).toBeInstanceOf(Context);
    expect(minimalCtx.asRaw()).not.toBe(0n);
  });

  test("Context.full() creates a valid Context instance", () => {
    const ctx = Context.full();
    expect(ctx).toBeInstanceOf(Context);
    expect(ctx.eval("'hello' + ' ' + 'world'")).toBe("hello world");
  });

  test("Context.builder() creates a valid Context instance", () => {
    const ctx = Context.builder();
    expect(ctx).toBeInstanceOf(Context);
    expect(ctx.asRaw()).not.toBe(0n);
  });

  test("Context.prototype methods exist and function correctly", () => {
    const ctx = Context.full();

    expect(typeof Context.prototype.asRaw).toBe("function");
    expect(typeof Context.prototype.getEnginePtr).toBe("function");
    expect(typeof Context.prototype.Engine).toBe("function");
    expect(typeof Context.prototype.with).toBe("function");

    const rawPtr = ctx.asRaw();
    expect(typeof rawPtr).toBe("bigint");
    expect(rawPtr).not.toBe(0n);

    const enginePtr = ctx.getEnginePtr();
    expect(typeof enginePtr).toBe("bigint");
    expect(enginePtr).not.toBe(0n);

    const engine = ctx.Engine();
    expect(engine).toBeInstanceOf(Engine);

    ctx.with((innerCtx: any) => {
      expect(innerCtx).toBeDefined();
      expect(innerCtx.eval("40 + 2")).toBe(42);
    });
  });

  test("Context.fromRaw() creates a Context instance from raw ptr and Engine", () => {
    const originalCtx = Context.full();
    const rawPtr = originalCtx.asRaw();
    const engine = originalCtx.Engine();

    const recreatedCtx = Context.fromRaw(rawPtr, engine);
    expect(recreatedCtx).toBeInstanceOf(Context);
    expect(recreatedCtx.eval("3 + 3")).toBe(6);
  });
});
