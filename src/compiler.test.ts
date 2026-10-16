import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler unit tests", () => {
  test("constructs Compiler instance", () => {
    const compiler = new Compiler();
    expect(compiler).toBeDefined();
  });

  test("compiles JavaScript into ESTree AST JSON string", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x = 42;", undefined, undefined);
    expect(typeof result).toBe("string");
    const parsed = JSON.parse(result) as { type: string; body: unknown[] };
    expect(parsed.type).toBe("Program");
    expect(Array.isArray(parsed.body)).toBe(true);
  });

  test("compiles TSX with explicit source_type and source_path", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const App = () => <div />;", "tsx", "src/App.tsx");
    expect(typeof result).toBe("string");
    const parsed = JSON.parse(result) as { type: string };
    expect(parsed.type).toBe("Program");
  });

  test("throws error when syntax error is encountered", () => {
    const compiler = new Compiler();
    expect(() => compiler.execute("const = ;", "js", "invalid.js")).toThrow();
  });
});
