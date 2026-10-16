/**
 * Tests for the native compiler bindings.
 */
import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("compiles valid JavaScript code into serialized ESTree JSON", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x = 42;", undefined, "index.js");
    expect(result).toBeDefined();
    const parsed = JSON.parse(result) as { type: string };
    expect(parsed.type).toBe("Program");
  });

  test("throws on syntax error during compilation", () => {
    const compiler = new Compiler();
    expect(() => compiler.execute("const = ;", undefined, "index.js")).toThrow();
  });
});
