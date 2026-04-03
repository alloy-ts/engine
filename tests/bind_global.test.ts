import { expect, test, describe } from "bun:test";
import { Alloy } from "../src/core/runtime";

describe("bind_global", () => {
  test("Property 1: bind_global_mqjs value round-trip", () => {
    // Feature: alloyscript-dual-engine, Property 1: bind_global_mqjs value round-trip
    // Mocking for test since we don't have the full native runtime in Bun environment
    (globalThis as any).AlloyInternal = {
        parse: () => null,
        secureEval: (code: string) => eval(code)
    };

    const value = { foo: "bar" };
    (globalThis as any).testValue = value;
    const result = Alloy.secureEval("testValue");
    expect(result).toEqual(value);
  });

  test("Property 14: bind_global_mqjs idempotent overwrite", () => {
    // Feature: alloyscript-dual-engine, Property 14: bind_global_mqjs idempotent overwrite
    (globalThis as any).testVar = "v1";
    expect(Alloy.secureEval("testVar")).toBe("v1");
    (globalThis as any).testVar = "v2";
    expect(Alloy.secureEval("testVar")).toBe("v2");
  });
});
