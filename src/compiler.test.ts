import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("compiles valid JavaScript code to ESTree JSON string", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const a = 10;", undefined, "index.js");
    expect(result).toBeDefined();
    expect(typeof result).toBe("string");
    const json = JSON.parse(result) as { type: string };
    expect(json.type).toBe("Program");
  });

  test("throws error on syntax error in source", () => {
    const compiler = new Compiler();
    expect(() => {
      compiler.execute("const a =", undefined, "index.js");
    }).toThrow();
  });
});
