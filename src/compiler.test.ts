/**
 * Tests for Compiler NAPI bindings.
 */
import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("constructs Compiler and executes parsing/compilation on JavaScript source", () => {
    const compiler = new Compiler();
    expect(compiler).toBeDefined();

    const output = compiler.execute("const a = 1 + 2;", "js", "index.js");
    expect(typeof output).toBe("string");
    const json = JSON.parse(output) as { type: string };
    expect(json.type).toBe("Program");
  });

  test("throws error when executing on invalid source text", () => {
    const compiler = new Compiler();
    expect(() => compiler.execute("const = ;", "js", "index.js")).toThrow();
  });
});
