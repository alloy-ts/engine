import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("new Compiler() instantiates Compiler", () => {
    const compiler = new Compiler();
    expect(compiler).toBeInstanceOf(Compiler);
  });

  test("execute parses valid JavaScript source code", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x = 42;");
    expect(typeof result).toBe("string");
    const parsed = JSON.parse(result);
    expect(parsed.type).toBe("Program");
    expect(parsed.body[0].type).toBe("VariableDeclaration");
  });

  test("execute supports source_type and source_path parameters", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x: number = 42;", "ts", "app.ts");
    expect(typeof result).toBe("string");
    const parsed = JSON.parse(result);
    expect(parsed.type).toBe("Program");
  });

  test("execute throws error on invalid syntax", () => {
    const compiler = new Compiler();
    expect(() => {
      compiler.execute("const x = ;");
    }).toThrow();
  });
});
