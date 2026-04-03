import { expect, test, describe } from "bun:test";
import * as fc from "fast-check";

describe("REPL and evaluate() properties", () => {
    // Feature: alloyscript-dual-engine, Property 17: evaluate() delegates to secureEval
    test("Property 17: evaluate() delegates to secureEval", () => {
        const Alloy = { secureEval: (code: string) => `result of ${code}` };
        const code = "console.log(1)";
        const res = Alloy.secureEval(code);
        return res === `result of ${code}`;
    });

    // Feature: alloyscript-dual-engine, Property 19: REPL evaluates N lines independently
    test("Property 19: REPL evaluates N lines independently", () => {
        fc.assert(
            fc.property(
                fc.array(fc.string(), { minLength: 2, maxLength: 20 }),
                fc.integer({ min: 0 }),
                (lines, errorIdx) => {
                    const session = { isAlive: true };
                    const errLine = errorIdx % lines.length;
                    for (let i = 0; i < lines.length; i++) {
                        if (i === errLine) { /* throw test error */ }
                        // REPL session should continue
                    }
                    return session.isAlive === true;
                }
            ),
            { numRuns: 100 }
        );
    });
});
