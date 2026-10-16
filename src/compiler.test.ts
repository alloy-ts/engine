/**
 * Tests for the `Compiler` NAPI bindings (`src/compiler.rs`).
 */
import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("Compiler constructor returns a Compiler instance", () => {
    const compiler = new Compiler();
    expect(compiler).toBeDefined();
  });

  test("execute parses valid JavaScript source and returns ESTree JSON", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x = 42;");
    expect(typeof result).toBe("string");
    const parsed = JSON.parse(result) as { type: string };
    expect(parsed.type).toBe("Program");
  });

  test("execute accepts sourceType and sourcePath options", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const a: number = 1;", "ts", "app.ts");
    expect(typeof result).toBe("string");
    const parsed = JSON.parse(result) as { type: string };
    expect(parsed.type).toBe("Program");
  });

  test("execute throws on invalid syntax", () => {
    const compiler = new Compiler();
    expect(() => {
      compiler.execute("const x = ;");
    }).toThrow();
  });
});
