import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("constructs Compiler instance", () => {
    const compiler = new Compiler();
    expect(compiler).toBeDefined();
  });

  test("executes compilation on JavaScript source and returns ESTree JSON", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x = 1;", undefined, "index.js");
    expect(typeof result).toBe("string");
    const json = JSON.parse(result) as { type: string };
    expect(json.type).toBe("Program");
  });

  test("supports TypeScript source with sourceType or sourcePath override", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x: number = 42;", "ts", "app.ts");
    expect(typeof result).toBe("string");
    const json = JSON.parse(result) as { type: string };
    expect(json.type).toBe("Program");
  });

  test("throws on invalid syntax during execution", () => {
    const compiler = new Compiler();
    expect(() => compiler.execute("const = ;", undefined, undefined)).toThrow();
  });
});
