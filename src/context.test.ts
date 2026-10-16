import { describe, expect, test } from "vite-plus/test";
import { Context, Runtime } from "./main.ts";

describe("napi: Context", () => {
  test("constructs Context with default constructor", () => {
    const ctx = new Context();
    expect(ctx).toBeDefined();
  });

  test("creates context via createBase and createFull factory methods", () => {
    const baseCtx = Context.createBase();
    expect(baseCtx).toBeDefined();

    const fullCtx = Context.createFull();
    expect(fullCtx).toBeDefined();
  });

  test("eval evaluates JavaScript expression", () => {
    const ctx = new Context();
    const result = ctx.eval("1 + 2");
    expect(result).toBe(3);
  });

  test("eval_with_options evaluates JavaScript with options", () => {
    const ctx = new Context();
    const result = ctx.evalWithOptions("2 * 21", { strict: true });
    expect(result).toBe(42);
  });

  test("json_parse parses JSON string", () => {
    const ctx = new Context();
    const res = ctx.jsonParse('{"a":123}') as { a: number };
    expect(res).toBeDefined();
    expect(res.a).toBe(123);
  });

  test("json_stringify stringifies JS value", () => {
    const ctx = new Context();
    const str = ctx.jsonStringify({ b: "hello" });
    expect(str).toBe('{"b":"hello"}');
  });

  test("has_exception returns false for valid context state", () => {
    const ctx = new Context();
    expect(ctx.hasException()).toBe(false);
  });

  test("run_gc runs garbage collection on runtime", () => {
    const ctx = new Context();
    expect(() => ctx.runGc()).not.toThrow();
  });
});
