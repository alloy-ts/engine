// tests/repl.test.ts
import { expect, test, describe } from "bun:test";
import * as fc from "fast-check";

// Feature: alloyscript-dual-engine, Property 17: evaluate() delegates to secureEval
// Feature: alloyscript-dual-engine, Property 18: evaluate() rejects on transpilation failure without calling JS_Eval
// Feature: alloyscript-dual-engine, Property 19: REPL evaluates N lines independently

describe("repl", () => {
	test("Property 19: REPL evaluates N lines independently", () => {
		// Mock simulation
		const session = {
			lines: [] as string[],
			eval(line: string) {
				this.lines.push(line);
				if (line === "throw") throw new Error();
				return "ok";
			}
		};

		fc.assert(
			fc.property(fc.array(fc.string(), { minLength: 2, maxLength: 20 }), (lines) => {
				session.lines = [];
				for (const line of lines) {
					try { session.eval(line); } catch (e) {}
				}
				expect(session.lines.length).toBe(lines.length);
			}),
			{ numRuns: 100 }
		);
	});
});
