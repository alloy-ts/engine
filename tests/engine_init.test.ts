import { expect, test, describe } from "bun:test";
import { Alloy } from "../src/core/runtime";

describe("engine_init", () => {
  test("Property 11: AlloyRuntime auto-binding completeness", () => {
    // Feature: alloyscript-dual-engine, Property 11: AlloyRuntime auto-binding completeness
    (globalThis as any).AlloyInternal = {
        parse: () => null,
        secureEval: (code: string) => eval(code)
    };
    // Mock globals that should be auto-bound
    (globalThis as any).fetch = () => {};
    (globalThis as any).crypto = {};
    (globalThis as any).URL = class {};
    (globalThis as any).TextEncoder = class {};
    (globalThis as any).TextDecoder = class {};
    (globalThis as any).performance = {};

    expect(Alloy.secureEval("typeof fetch")).not.toBe("undefined");
    expect(Alloy.secureEval("typeof crypto")).not.toBe("undefined");
    expect(Alloy.secureEval("typeof URL")).not.toBe("undefined");
    expect(Alloy.secureEval("typeof TextEncoder")).not.toBe("undefined");
    expect(Alloy.secureEval("typeof TextDecoder")).not.toBe("undefined");
    expect(Alloy.secureEval("typeof performance")).not.toBe("undefined");
    expect(Alloy.secureEval("typeof Alloy.secureEval")).not.toBe("undefined");
    expect(Alloy.secureEval("typeof Alloy.Transpiler")).not.toBe("undefined");
  });
});
