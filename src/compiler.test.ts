import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("constructs Compiler instance", () => {
    const compiler = new Compiler();
    expect(compiler).toBeDefined();
  });

  test("executes compilation on valid JavaScript code", () => {
    const compiler = new Compiler();
    const resultJson = compiler.execute("const a = 10;", undefined, undefined);
    expect(resultJson).toBeDefined();
    const ast = JSON.parse(resultJson) as { type: string };
    expect(ast.type).toBe("Program");
  });

  test("executes compilation with source_type and source_path options", () => {
    const compiler = new Compiler();
    const resultJson = compiler.execute(
      "const el = <div />;",
      "tsx",
      "src/Component.tsx",
    );
    expect(resultJson).toBeDefined();
    const ast = JSON.parse(resultJson) as { type: string };
    expect(ast.type).toBe("Program");
  });

  test("throws error on syntax diagnostics", () => {
    const compiler = new Compiler();
    expect(() => {
      compiler.execute("const = ;", undefined, undefined);
    }).toThrow();
  });
});
