import { describe, expect, test } from "vite-plus/test";
import { Context, Engine } from "./main.ts";

describe("napi: Context constructors and methods", () => {
  test("Context static constructors: base, custom, full, builder, fromRaw", () => {
    const engine = new Engine();

    const ctxBase = Context.base(engine);
    expect(ctxBase).toBeDefined();

    const ctxCustom = Context.custom(engine);
    expect(ctxCustom).toBeDefined();

    const ctxFull = Context.full(engine);
    expect(ctxFull).toBeDefined();

    const ctxBuilder = Context.builder();
    expect(ctxBuilder).toBeDefined();

    const ctxFromRaw = Context.fromRaw(engine);
    expect(ctxFromRaw).toBeDefined();
  });

  test("Context prototype methods: asRaw, getEnginePtr, Engine, with", () => {
    const ctx = Context.new();

    expect(typeof ctx.asRaw()).toBe("string");
    expect(typeof ctx.as_raw()).toBe("string");

    expect(typeof ctx.getEnginePtr()).toBe("string");
    expect(typeof ctx.get_engine_ptr()).toBe("string");

    const rt = ctx.Engine();
    expect(rt).toBeDefined();

    const res = ctx.with(() => 42);
    expect(res).toBe(42);
  });
});
