import { describe, expect, test } from "vite-plus/test";
import { Context, Engine, JSContext } from "./main.ts";

describe("napi: Context & Engine bindings", () => {
  test("creates Context with full, base, custom, builder, and fromRaw constructors", () => {
    const engine = new Engine();
    const ctxFull = Context.full(engine);
    expect(ctxFull).toBeDefined();

    const ctxBase = Context.base(engine);
    expect(ctxBase).toBeDefined();

    const ctxCustom = Context.custom(engine);
    expect(ctxCustom).toBeDefined();

    const ctxBuilder = Context.builder(engine);
    expect(ctxBuilder).toBeDefined();

    const rawPtr = ctxFull.asRaw();
    expect(typeof rawPtr).toBe("bigint");
    expect(rawPtr).toBeGreaterThan(0n);

    const ctxFromRaw = Context.fromRaw(rawPtr, engine);
    expect(ctxFromRaw).toBeDefined();
  });

  test("prototype methods: asRaw, getEnginePtr, Engine, and with", () => {
    const engine = new Engine();
    const ctx = Context.full(engine);

    const rawPtr = ctx.asRaw();
    expect(typeof rawPtr).toBe("bigint");

    const enginePtr = ctx.getEnginePtr();
    expect(typeof enginePtr).toBe("bigint");

    const eng = ctx.Engine();
    expect(eng).toBeDefined();

    let called = false;
    ctx.with((c: unknown) => {
      called = true;
      expect(c).toBeDefined();
    });
    expect(called).toBe(true);
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
    expect(res).toEqual({ value: 42 });
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
});
