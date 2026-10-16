/**
 * Tests for Context, Runtime, and Loader NAPI bindings.
 */
import { describe, expect, test } from "vite-plus/test";
import { Context, Loader, Runtime } from "./main.ts";

describe("napi: Runtime, Context & Loader", () => {
  test("constructs Runtime and sets limits", () => {
    const runtime = new Runtime();
    expect(runtime).toBeDefined();

    runtime.setMemoryLimit(1024 * 1024 * 16);
    runtime.setMaxStackSize(1024 * 1024);
    runtime.runGc();
  });

  test("constructs Loader and initializes Runtime with Loader", () => {
    const loader = new Loader();
    loader.addModule("my_mod", "export const foo = 42;");

    const runtime = new Runtime(loader);
    expect(runtime).toBeDefined();

    const context = Context.full(runtime);
    expect(context).toBeDefined();
  });

  test("constructs Context with default Runtime and evaluates JS", () => {
    const context = new Context(undefined);
    expect(context).toBeDefined();

    const result = context.eval("1 + 2");
    expect(result).toBe("3");
  });

  test("constructs Context with explicit Runtime using full and base constructors", () => {
    const runtime = new Runtime();
    const fullCtx = Context.full(runtime);
    expect(fullCtx).toBeDefined();

    const fullResult = fullCtx.eval("({ a: 1 })");
    expect(JSON.parse(fullResult)).toEqual({ a: 1 });

    const baseCtx = Context.base(runtime);
    expect(baseCtx).toBeDefined();

    fullCtx.runGc();
    baseCtx.runGc();
  });
});
