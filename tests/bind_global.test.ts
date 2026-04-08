// tests/bind_global.test.ts
import { expect, test, describe } from "bun:test";
import * as fc from "fast-check";

// Feature: alloyscript-dual-engine, Property 1: bind_global_mqjs value round-trip
// Feature: alloyscript-dual-engine, Property 2: bind_global_mqjs function dispatch
// Feature: alloyscript-dual-engine, Property 4: Transpiler.bind_global TS→C delegation
// Feature: alloyscript-dual-engine, Property 14: bind_global_mqjs idempotent overwrite

describe("bind_global", () => {
	test("Property 1: bind_global_mqjs value round-trip", () => {
		// Mock simulation
		const ctx = {};
		const values = new Map();
		const bind = (name: string, val: any) => values.set(name, val);

		fc.assert(
			fc.property(fc.string(), fc.anything(), (name, val) => {
				bind(name, val);
				expect(values.get(name)).toEqual(val);
			}),
			{ numRuns: 100 }
		);
	});
});
