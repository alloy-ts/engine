/**
 * Tests for Context, Engine, and Loader NAPI bindings.
 */
import { describe, expect, test } from "vite-plus/test";
import { Context, Engine, Loader } from "./main.ts";

describe("napi: Context & Engine", () => {
  test("constructs Engine and sets limits", () => {
    const engine = new Engine();
    expect(engine).toBeDefined();

    engine.setMemoryLimit(1024 * 1024 * 16);
    engine.setMaxStackSize(1024 * 1024);
    engine.runGc();
  });

  test("constructs Loader and initializes Engine with Loader", () => {
    const loader = new Loader();
    loader.addModule("my_mod", "export const foo = 42;");

    const engine = new Engine(loader);
    expect(engine).toBeDefined();

    const context = Context.full(engine);
    expect(context).toBeDefined();
  });

  test("constructs Context with default Engine and evaluates JS", () => {
    const context = new Context(undefined);
    expect(context).toBeDefined();

    const result = context.eval("1 + 2");
    expect(result).toBe("3");
  });

  test("constructs Context with explicit Engine using full and base constructors", () => {
    const engine = new Engine();
    const fullCtx = Context.full(engine);
    expect(fullCtx).toBeDefined();

    const fullResult = fullCtx.eval("({ a: 1 })");
    expect(JSON.parse(fullResult)).toEqual({ a: 1 });

    const baseCtx = Context.base(engine);
    expect(baseCtx).toBeDefined();

    fullCtx.runGc();
    baseCtx.runGc();
  });
});
