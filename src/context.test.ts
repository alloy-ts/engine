import { describe, expect, test } from "vite-plus/test";
import { Context, Engine, JSContext, Runtime } from "./main.ts";

describe("napi: Runtime & Context bindings", () => {
  test("creates a Runtime and Configures limits", () => {
    const rt = new Runtime();
    expect(rt).toBeDefined();

    rt.set_info("test-runtime");
    rt.set_memory_limit(1024 * 1024);
    rt.set_max_stack_size(256 * 1024);
    rt.set_gc_threshold(1000);
    rt.run_gc();

    const mem = rt.memory_usage();
    expect(mem).toBeDefined();
    expect(mem.malloc_size).toBeGreaterThanOrEqual(0);
  });

  test("creates Context with full and base constructors", () => {
    const rt = new Runtime();
    const ctxFull = Context.full(undefined, rt);
    expect(ctxFull).toBeDefined();

    const ctxBase = Context.base(undefined, rt);
    expect(ctxBase).toBeDefined();

    const defaultCtx = new Context();
    expect(defaultCtx).toBeDefined();
  });

  test("creates Context with an Engine instance", () => {
    const engine = new Engine();
    const ctx = Context.full(engine);
    expect(ctx).toBeDefined();
    expect(ctx.eval("10 * 10")).toBe(100);
  });

  test("JSContext alias is identical to Context", () => {
    expect(JSContext).toBe(Context);
  });

  test("evaluates expressions and returns values", () => {
    const ctx = new Context();
    expect(ctx.eval("1 + 1")).toBe(2);
    expect(ctx.eval("'hello' + ' world'")).toBe("hello world");
    expect(ctx.eval("({ a: 1, b: 'two' })")).toEqual({ a: 1, b: "two" });
  });

  test("evaluates with options", () => {
    const ctx = new Context();
    const res = ctx.eval("function f() { return 42; }; f();", {
      strict: true,
      filename: "test.js",
    });
    expect(res).toBe(42);
  });

  test("compiles valid script without executing side-effects", () => {
    const ctx = new Context();
    expect(() => ctx.compile("let a = 1;", "test.js", true)).not.toThrow();
    // Verify side-effect didn't run: 'a' is not defined in global context
    expect(() => ctx.eval("a")).toThrow();
  });

  test("throws on compile syntax error", () => {
    const ctx = new Context();
    expect(() => ctx.compile("let = ;", "test.js", true)).toThrow();
  });

  test("evaluates promise and resolves value", () => {
    const ctx = new Context();
    const res = ctx.eval_promise("await Promise.resolve(42)");
    expect(res).toBe(42);
  });

  test("throws on eval exception", () => {
    const ctx = new Context();
    expect(() => ctx.eval("throw new Error('boom')")).toThrow("boom");
  });

  test("json_parse and json_stringify work as expected", () => {
    const ctx = new Context();
    const parsed = ctx.json_parse('{"x": 100}');
    expect(parsed).toEqual({ x: 100 });

    const stringified = ctx.json_stringify({ y: 200 });
    expect(stringified).toBe('{"y":200}');
  });

  test("gets underlying runtime from context", () => {
    const ctx = new Context();
    const rt = ctx.get_runtime();
    expect(rt).toBeDefined();
    expect(rt.is_job_pending()).toBe(false);
  });
});
