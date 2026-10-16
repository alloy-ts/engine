import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("Compiler can be instantiated and executes valid source code", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const a = 1;");
    expect(typeof result).toBe("string");
    const ast = JSON.parse(result);
    expect(ast.type).toBe("Program");
  });

  test("Compiler handles sourceType and sourcePath options", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x: number = 42;", "ts", "app.ts");
    expect(typeof result).toBe("string");
    const ast = JSON.parse(result);
    expect(ast.type).toBe("Program");
  });

  test("Compiler throws an error on invalid syntax", () => {
    const compiler = new Compiler();
    expect(() => {
      compiler.execute("const = ;");
    }).toThrow();
  });
});
