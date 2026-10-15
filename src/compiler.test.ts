import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("constructs Compiler instance", () => {
    const compiler = new Compiler();
    expect(compiler).toBeDefined();
  });

  test("compiles JavaScript source into serialized ESTree JSON string", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x = 1;", undefined, "index.js");
    expect(typeof result).toBe("string");
    const parsed = JSON.parse(result) as { type: string };
    expect(parsed.type).toBe("Program");
  });

  test("infers source type from file extension", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const el = <div />;", undefined, "app.tsx");
    expect(typeof result).toBe("string");
    const parsed = JSON.parse(result) as { type: string };
    expect(parsed.type).toBe("Program");
  });

  test("throws on invalid JavaScript source", () => {
    const compiler = new Compiler();
    expect(() => compiler.execute("const = ;", undefined, "index.js")).toThrow();
  });
});
