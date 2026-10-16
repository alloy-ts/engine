import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("constructs a new Compiler instance", () => {
    const compiler = new Compiler();
    expect(compiler).toBeDefined();
  });

  test("executes compilation on valid JavaScript code", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x = 42;", undefined, "app.js");
    expect(typeof result).toBe("string");

    const ast = JSON.parse(result);
    expect(ast.type).toBe("Program");
  });

  test("executes compilation on TypeScript code with source_type", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const y: number = 100;", "ts", "app.ts");
    expect(typeof result).toBe("string");

    const ast = JSON.parse(result);
    expect(ast.type).toBe("Program");
  });

  test("throws error on syntax failure", () => {
    const compiler = new Compiler();
    expect(() => compiler.execute("const = ;", "js", "app.js")).toThrow();
  });
});
