import { describe, expect, test } from "vite-plus/test";
import { Context } from "./main.ts";

describe("napi: Context", () => {
  test("constructs Context instance", () => {
    const ctx = new Context();
    expect(ctx).toBeDefined();
  });

  test("create_base and create_full factory methods work", () => {
    const baseCtx = Context.create_base();
    expect(baseCtx).toBeDefined();

    const fullCtx = Context.create_full();
    expect(fullCtx).toBeDefined();
  });

  test("eval evaluates JavaScript expression", () => {
    const ctx = new Context();
    const result = ctx.eval("1 + 2");
    expect(result).toBe(3);
  });

  test("eval evaluates object / array expressions", () => {
    const ctx = new Context();
    const result = ctx.eval("({ a: 1, b: 'hello', c: [true, false] })");
    expect(result).toEqual({ a: 1, b: "hello", c: [true, false] });
  });

  test("eval with options", () => {
    const ctx = new Context();
    const result = ctx.eval("function test() { foo = 'bar'; return foo; } test()", {
      strict: false,
    });
    expect(result).toBe("bar");
  });

  test("eval throws error on JavaScript exception", () => {
    const ctx = new Context();
    expect(() => ctx.eval("throw new Error('something went wrong')")).toThrow(
      /something went wrong/,
    );
  });

  test("json_parse parses JSON string", () => {
    const ctx = new Context();
    const parsed = ctx.json_parse('{"num": 42, "str": "ok"}');
    expect(parsed).toEqual({ num: 42, str: "ok" });
  });

  test("json_stringify stringifies value", () => {
    const ctx = new Context();
    const str = ctx.json_stringify({ name: "alloy", version: 1 });
    expect(str).toBe('{"name":"alloy","version":1}');
  });

  test("execute_pending_job and run_gc execute without errors", () => {
    const ctx = new Context();
    const executed = ctx.execute_pending_job();
    expect(typeof executed).toBe("boolean");
    expect(() => ctx.run_gc()).not.toThrow();
  });
});
