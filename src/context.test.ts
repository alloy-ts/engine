import { describe, expect, test } from "vite-plus/test";
import { Context } from "./main.ts";

describe("napi: Context", () => {
  test("constructs Context via new, base(), and full()", () => {
    const ctx1 = new Context();
    expect(ctx1).toBeDefined();

    const ctx2 = Context.base();
    expect(ctx2).toBeDefined();

    const ctx3 = Context.full();
    expect(ctx3).toBeDefined();
  });

  test("eval evaluates JavaScript expression", () => {
    const ctx = new Context();
    const result = ctx.eval("1 + 1");
    expect(result).toBe(2);
  });

  test("compile checks code syntax", () => {
    const ctx = new Context();
    expect(ctx.compile("const x = 1;", "test.js", true)).toBe(true);
  });

  test("evalPromise evaluates async/promise expression", () => {
    const ctx = new Context();
    const result = ctx.evalPromise("2 * 3");
    expect(result).toBe(6);
  });

  test("jsonParse and jsonStringify handle JSON data", () => {
    const ctx = new Context();
    const parsed = ctx.jsonParse('{"hello":"world"}') as { hello: string };
    expect(parsed.hello).toBe("world");

    const jsonStr = ctx.jsonStringify({ key: "value" });
    expect(jsonStr).toBe('{"key":"value"}');
  });

  test("hasException returns false when no exception is pending", () => {
    const ctx = new Context();
    expect(ctx.hasException()).toBe(false);
  });

  test("runGc executes garbage collection", () => {
    const ctx = new Context();
    expect(() => ctx.runGc()).not.toThrow();
  });
});
