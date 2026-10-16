import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("constructs Compiler instance", () => {
    const compiler = new Compiler();
    expect(compiler).toBeDefined();
  });

  test("executes compilation on valid source text returning ESTree JSON string", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x = 10;", undefined, "app.ts");
    expect(typeof result).toBe("string");
    const json = JSON.parse(result);
    expect(json.type).toBe("Program");
  });

  test("throws error on syntax error", () => {
    const compiler = new Compiler();
    expect(() => compiler.execute("const = ;", undefined, "app.ts")).toThrow();
  });
});
