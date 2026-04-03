import { expect, test, describe } from "bun:test";
import { Transpiler } from "../src/core/transpiler";

describe("transpiler", () => {
  test("Property 24: Async/await rewrite removes async keyword", () => {
    // Feature: alloyscript-dual-engine, Property 24: Async/await rewrite removes async keyword
    const transpiler = new Transpiler();
    const code = "async function foo() { await bar(); }";
    const result = transpiler.transformSync(code);
    expect(result).not.toContain("async function");
    expect(result).toContain("function*");
    expect(result).toContain("yield");
  });

  test("Property 25: Promise rewrite replaces new Promise with __alloy_deferred", () => {
    // Feature: alloyscript-dual-engine, Property 25: Promise rewrite replaces new Promise with __alloy_deferred
    const transpiler = new Transpiler();
    const code = "new Promise((resolve) => resolve())";
    const result = transpiler.transformSync(code);
    expect(result).not.toContain("new Promise(");
    expect(result).toContain("__alloy_deferred(");
  });
});
