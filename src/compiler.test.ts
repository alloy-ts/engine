/**
 * Unit tests for the oxc-backed Compiler class (`src/compiler.rs`).
 */
import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("new Compiler() creates an instance and executes valid code", () => {
    const compiler = new Compiler();
    expect(compiler).toBeDefined();

    const output = compiler.execute("const a = 10;", "js", "index.js");
    expect(typeof output).toBe("string");
    expect(output).toContain("Program");
    expect(output).toContain("VariableDeclaration");
  });

  test("Compiler.execute correctly parses TypeScript code with source_type override", () => {
    const compiler = new Compiler();
    const output = compiler.execute("const x: number = 42;", "ts", "app.ts");
    expect(output).toContain("Program");
    expect(output).toContain("VariableDeclaration");
  });

  test("Compiler.execute throws an error on syntax errors", () => {
    const compiler = new Compiler();
    expect(() => {
      compiler.execute("const const = ;", "js", "invalid.js");
    }).toThrow();
  });

  test("Compiler.execute works with default parameters", () => {
    const compiler = new Compiler();
    const output = compiler.execute("let y = 'hello';", undefined, undefined);
    expect(output).toContain("Program");
  });
});
