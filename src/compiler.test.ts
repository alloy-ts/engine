import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("Compiler", () => {
  test("should instantiate Compiler", () => {
    const compiler = new Compiler();
    expect(compiler).toBeDefined();
  });

  test("should execute compilation and return AST JSON", () => {
    const compiler = new Compiler();
    const code = "const x = 42;";
    const result = compiler.execute(code);
    expect(typeof result).toBe("string");

    const ast = JSON.parse(result);
    expect(ast.type).toBe("Program");
    expect(ast.body).toHaveLength(1);
  });

  test("should accept custom source_type and source_path", () => {
    const compiler = new Compiler();
    const code = "const a: number = 100;";
    const result = compiler.execute(code, "ts", "app.ts");
    expect(typeof result).toBe("string");

    const ast = JSON.parse(result);
    expect(ast.type).toBe("Program");
  });

  test("should throw error on invalid syntax", () => {
    const compiler = new Compiler();
    const code = "const x =";
    expect(() => compiler.execute(code)).toThrow();
  });
});
