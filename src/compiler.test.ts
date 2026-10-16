import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("compiles valid JavaScript source into ESTree JSON", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x = 42;", undefined, undefined);
    expect(result).toBeDefined();
    const ast = JSON.parse(result) as { type: string };
    expect(ast.type).toBe("Program");
  });

  test("compiles TypeScript source with sourcePath", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x: number = 42;", undefined, "src/index.ts");
    expect(result).toBeDefined();
    const ast = JSON.parse(result) as { type: string };
    expect(ast.type).toBe("Program");
  });

  test("throws on invalid JavaScript syntax", () => {
    const compiler = new Compiler();
    expect(() => compiler.execute("const = ;", undefined, undefined)).toThrow();
  });
});
