import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("compiles valid JavaScript code to ESTree JSON string", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x = 1;", undefined, "app.js");
    expect(typeof result).toBe("string");
    const parsed = JSON.parse(result) as { type: string };
    expect(parsed.type).toBe("Program");
  });

  test("throws on syntax error during compilation", () => {
    const compiler = new Compiler();
    expect(() => compiler.execute("const = ;", undefined, "app.js")).toThrow();
  });
});
