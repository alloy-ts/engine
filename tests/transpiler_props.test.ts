import { expect, test } from "bun:test";
import * as fc from "fast-check";
import { Transpiler } from "../src/core/transpiler";

// Feature: alloyscript-dual-engine, Property 24: Async/await rewrite removes async keyword
test("Property 24: Async/await rewrite removes async keyword", () => {
    const transpiler = new Transpiler();
    fc.assert(
        fc.property(fc.string({ minLength: 1, maxLength: 100 }), (src) => {
            const safe = src.replace(/[^a-z]/gi, '_');
            const code = `async function f() { await ${safe} }`;
            const out = transpiler.transformSync(code, "ts");
            return !out.includes("async ") && (out.includes("function*") || out.includes("yield"));
        }),
        { numRuns: 100 }
    );
});

// Feature: alloyscript-dual-engine, Property 25: Promise rewrite replaces new Promise
test("Property 25: Promise rewrite replaces new Promise", () => {
    const transpiler = new Transpiler();
    fc.assert(
        fc.property(fc.string({ minLength: 1, maxLength: 100 }), (src) => {
            const code = `new Promise((r) => r(${JSON.stringify(src)}))`;
            const out = transpiler.transformSync(code, "ts");
            return out.includes("__alloy_deferred(") && !out.includes("new Promise(");
        }),
        { numRuns: 100 }
    );
});

// Feature: alloyscript-dual-engine, Property 3: MQJS compatibility (var instead of const/let)
test("Property 3: MQJS compatibility rewrites const/let to var", () => {
    const transpiler = new Transpiler();
    fc.assert(
        fc.property(fc.string({ minLength: 1, maxLength: 50 }), (src) => {
            const safe = src.replace(/[^a-z]/gi, '_');
            const code = `const c_${safe} = 1; let l_${safe} = 2;`;
            const out = transpiler.transformSync(code, "ts");
            return out.includes(`var c_${safe}`) && out.includes(`var l_${safe}`) && !out.includes("const ") && !out.includes("let ");
        }),
        { numRuns: 100 }
    );
});
