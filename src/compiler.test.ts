import { describe, expect, it } from "vitest";
import { Compiler } from "./main";

describe("Compiler", () => {
  it("compiles JavaScript source code into ESTree JSON", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x = 42;", "js", "test.js");
    expect(result).toBeDefined();
    expect(typeof result).toBe("string");
    const json = JSON.parse(result);
    expect(json.type).toBe("Program");
  });

  it("throws an error on invalid syntax", () => {
    const compiler = new Compiler();
    expect(() => {
      compiler.execute("const x =", "js", "test.js");
    }).toThrow();
  });
});
