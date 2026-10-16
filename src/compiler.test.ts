import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("executes compilation on source text and returns ESTree JSON string", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x = 1;", undefined, undefined);

    expect(typeof result).toBe("string");
    const parsed = JSON.parse(result) as { type: string };
    expect(parsed.type).toBe("Program");
  });

  test("throws on invalid syntax", () => {
    const compiler = new Compiler();
    expect(() => compiler.execute("const = ;", undefined, undefined)).toThrow();
  });
});
