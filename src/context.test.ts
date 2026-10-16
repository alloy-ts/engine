import { describe, expect, test } from "vite-plus/test";
import { Context } from "./main.ts";

describe("napi: Context", () => {
  test("constructs Context and evaluates code", () => {
    const ctx = new Context();
    expect(ctx.eval("1 + 1")).toBe("2");
  });

  test("creates base Context", () => {
    const ctx = Context.createBase();
    expect(ctx).toBeDefined();
  });
});
