import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("new Compiler() creates an instance", () => {
    const compiler = new Compiler();
    expect(compiler).toBeDefined();
  });

  test("execute compiles valid JavaScript into ESTree JSON string", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x = 42;");
    expect(typeof result).toBe("string");
    const parsed = JSON.parse(result);
    expect(parsed.type).toBe("Program");
    expect(parsed.body).toBeDefined();
    expect(parsed.body.length).toBe(1);
    expect(parsed.body[0].type).toBe("VariableDeclaration");
  });

  test("execute accepts custom source_type and source_path parameters", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const a: number = 10;", "ts", "src/index.ts");
    expect(typeof result).toBe("string");
    const parsed = JSON.parse(result);
    expect(parsed.type).toBe("Program");
  });

  test("execute throws on syntax errors", () => {
    const compiler = new Compiler();
    expect(() => {
      compiler.execute("const = ;");
    }).toThrow();
  });
});
