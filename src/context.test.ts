import { describe, expect, test } from "vite-plus/test";
import { Context } from "./main.ts";

describe("napi: Context", () => {
  test("constructs a new Context", () => {
    const ctx = new Context();
    expect(ctx).toBeDefined();
  });

  test("evaluates simple JavaScript expressions", () => {
    const ctx = new Context();
    const res = ctx.eval("1 + 1");
    expect(res).toBe("2");
  });

  test("returns error on invalid JavaScript syntax", () => {
    const ctx = new Context();
    expect(() => ctx.eval("1 +")).toThrow();
  });
});
