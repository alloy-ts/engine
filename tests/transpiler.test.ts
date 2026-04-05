import { expect, test } from "bun:test";
import * as fc from "fast-check";
import { Transpiler } from "../src/core/transpiler.ts";

// Feature: alloyscript-dual-engine, Property 24: Async/await rewrite removes async keyword
test("Property 24: Async/await rewrite removes async keyword", () => {
    const transpiler = new Transpiler();
    fc.assert(
        fc.property(fc.string(), (src) => {
            const code = "async function foo() { await bar(); }";
            const out = transpiler.transformSync(code, "ts");
            return !out.includes("async ") && (out.includes("function*") || out.includes("yield"));
        }),
        { numRuns: 100 }
    );
});

// Feature: alloyscript-dual-engine, Property 25: Promise rewrite replaces new Promise
test("Property 25: Promise rewrite replaces new Promise", () => {
    const transpiler = new Transpiler();
    const code = "const p = new Promise((resolve) => resolve());";
    const out = transpiler.transformSync(code, "ts");
    expect(out).toContain("__alloy_deferred(");
    expect(out).not.toContain("new Promise(");
});
