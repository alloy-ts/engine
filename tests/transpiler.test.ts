import { describe, it, expect } from "bun:test";
import * as fc from "fast-check";
import { AlloyTranspiler } from "../src/core/transpiler";

// Feature: alloyscript-dual-engine, Property 24: Async/await rewrite removes async keyword
// Feature: alloyscript-dual-engine, Property 25: Promise rewrite replaces new Promise with __alloy_deferred
// Feature: alloyscript-dual-engine, Property 26: __alloy_await helper is injected into async-containing output

describe("Transpiler Properties", () => {
  const transpiler = new AlloyTranspiler();

  it("should remove async/await keywords", () => {
    const code = "async function foo() { await bar(); }";
    const output = transpiler.transformSync(code, "ts");
    expect(output).not.toContain("async ");
    expect(output).not.toContain("await bar");
    expect(output).toContain("function*");
    expect(output).toContain("yield");
  });

  it("should replace new Promise with __alloy_deferred", () => {
    const code = "const p = new Promise((resolve) => resolve());";
    const output = transpiler.transformSync(code, "ts");
    expect(output).not.toContain("new Promise");
    expect(output).toContain("__alloy_deferred");
  });
});
