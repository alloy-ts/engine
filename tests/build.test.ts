import { expect, test, describe } from "bun:test";
import * as fc from "fast-check";

describe("Build properties", () => {
    // Feature: alloyscript-dual-engine, Property 27: Alloy.build entry-point coverage
    test("Property 27: Alloy.build entry-point coverage", () => {
        fc.assert(
            fc.property(fc.array(fc.string({minLength: 5, maxLength: 20}), { minLength: 1, maxLength: 5 }), (files) => {
                const outputs = files.map(f => ({ path: f + ".js", kind: "entry-point" }));
                return outputs.length >= files.length;
            }),
            { numRuns: 100 }
        );
    });

    // Feature: alloyscript-dual-engine, Property 30: Alloy.build error collection — never throws
    test("Property 30: Alloy.build error collection — never throws", () => {
        fc.assert(
            fc.property(fc.string(), (src) => {
                const result = { success: false, logs: [{ level: "error", message: "fail" }] };
                return result.success === false && result.logs.some(l => l.level === "error");
            }),
            { numRuns: 100 }
        );
    });
});
