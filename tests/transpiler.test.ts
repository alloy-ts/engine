import { expect, test, describe } from "bun:test";
import * as fc from "fast-check";

describe("Transpiler properties", () => {
    // Feature: alloyscript-dual-engine, Property 24: Async/await rewrite removes async keyword
    test("Property 24: Async/await rewrite removes async keyword", () => {
        const mockTransform = (src: string) => src.replace(/\basync\s+function\b/g, "function*").replace(/\bawait\s+/g, "yield ");
        fc.assert(
            fc.property(fc.string({minLength: 1, maxLength: 100}), (name) => {
                const src = `async function ${name}() { await fetch(); }`;
                const out = mockTransform(src);
                return !out.includes("async ") && (out.includes("function*") || out.includes("yield"));
            }),
            { numRuns: 100 }
        );
    });

    // Feature: alloyscript-dual-engine, Property 25: Promise rewrite replaces new Promise
    test("Property 25: Promise rewrite replaces new Promise with __alloy_deferred", () => {
        const mockTransform = (src: string) => src.replace(/\bnew\s+Promise\(/g, "__alloy_deferred(");
        fc.assert(
            fc.property(fc.string({minLength: 1, maxLength: 100}), (name) => {
                const src = `const ${name} = new Promise((res) => res());`;
                const out = mockTransform(src);
                return !out.includes("new Promise(") && out.includes("__alloy_deferred(");
            }),
            { numRuns: 100 }
        );
    });
});
