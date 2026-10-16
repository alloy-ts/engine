import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("constructs a new Compiler instance", () => {
    const compiler = new Compiler();
    expect(compiler).toBeDefined();
  });

  test("executes compilation on JavaScript code and returns AST string", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const a = 1;", undefined, "app.js");
    expect(typeof result).toBe("string");
    expect(result).toContain("Program");
  });

  test("throws error on syntax error during compilation", () => {
    const compiler = new Compiler();
    expect(() => compiler.execute("const = ;", undefined, undefined)).toThrow();
  });
});
