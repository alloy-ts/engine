import { describe, expect, test } from "vite-plus/test";
import { Compiler } from "./main.ts";

describe("napi: Compiler", () => {
  test("compiles valid JavaScript code to ESTree JSON string", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x = 123;", undefined, "index.js");
    expect(typeof result).toBe("string");
    expect(result).toContain("Program");
  });

  test("compiles TypeScript and TSX code when source_type or source_path is supplied", () => {
    const compiler = new Compiler();
    const tsxResult = compiler.execute("const App = () => <div />;", "tsx", "src/App.tsx");
    expect(tsxResult).toContain("JSXElement");

    const tsResult = compiler.execute("const val: number = 42;", undefined, "src/math.ts");
    expect(tsResult).toContain("Program");
    expect(tsResult).toContain("val");
  });

  test("throws on syntax error during compilation", () => {
    const compiler = new Compiler();
    expect(() => compiler.execute("const = ;", undefined, "index.js")).toThrow();
  });
});
