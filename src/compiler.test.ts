import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("new Compiler() constructs a compiler instance", () => {
    const compiler = new Compiler();
    expect(compiler).toBeDefined();
    expect(typeof compiler.execute).toBe("function");
  });

  test("execute returns valid ESTree AST JSON string for valid JavaScript", () => {
    const compiler = new Compiler();
    const source = "const a = 1 + 2;";
    const result = compiler.execute(source);
    expect(typeof result).toBe("string");

    const ast = JSON.parse(result);
    expect(ast.type).toBe("Program");
    expect(Array.isArray(ast.body)).toBe(true);
    expect(ast.body.length).toBeGreaterThan(0);
  });

  test("execute handles TypeScript/JSX code with source_path option", () => {
    const compiler = new Compiler();
    const tsCode = "const greet = (name: string): string => `Hello ${name}`;";
    const result = compiler.execute(tsCode, undefined, "index.ts");
    expect(typeof result).toBe("string");

    const ast = JSON.parse(result);
    expect(ast.type).toBe("Program");
  });

  test("execute handles explicit source_type option", () => {
    const compiler = new Compiler();
    const moduleCode = "import { foo } from './foo'; export const bar = foo;";
    const result = compiler.execute(moduleCode, "module", "app.js");
    expect(typeof result).toBe("string");

    const ast = JSON.parse(result);
    expect(ast.type).toBe("Program");
    expect(ast.sourceType).toBe("module");
  });

  test("execute throws an error on syntax errors", () => {
    const compiler = new Compiler();
    const invalidCode = "const = ;";
    expect(() => {
      compiler.execute(invalidCode);
    }).toThrow();
  });
});
