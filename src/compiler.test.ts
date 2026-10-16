import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("constructs Compiler instance", () => {
    const compiler = new Compiler();
    expect(compiler).toBeDefined();
  });

  test("compiles valid JavaScript code to ESTree JSON string", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const a = 42;");
    expect(typeof result).toBe("string");
    const json = JSON.parse(result);
    expect(json.type).toBe("Program");
  });

  test("compiles TS code with source_type or source_path", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x: number = 10;", "ts", "app.ts");
    expect(typeof result).toBe("string");
    const json = JSON.parse(result);
    expect(json.type).toBe("Program");
  });

  test("throws error when syntax error occurs", () => {
    const compiler = new Compiler();
    expect(() => compiler.execute("const = ;")).toThrow();
  });
});
