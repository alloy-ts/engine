/**
 * Tests for Compiler bindings.
 */
import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("constructs a new Compiler instance", () => {
    const compiler = new Compiler();
    expect(compiler).toBeDefined();
  });

  test("compiles valid JS source code", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x = 1;", undefined, undefined);
    expect(result).toBeDefined();
    expect(typeof result).toBe("string");
    expect(result).toContain("Program");
  });

  test("infers source type from path or source_type parameter", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const a: number = 42;", "ts", "src/file.ts");
    expect(result).toBeDefined();
    expect(typeof result).toBe("string");
  });

  test("throws on syntax error during compilation", () => {
    const compiler = new Compiler();
    expect(() => compiler.execute("const = ;", undefined, undefined)).toThrow();
  });
});
