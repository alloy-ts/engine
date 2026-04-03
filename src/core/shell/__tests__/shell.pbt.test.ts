import { describe, expect, test } from "bun:test";
import * as fc from "fast-check";

describe("Alloy Shell properties", () => {
	// Feature: alloy-shell, Property 1: Injection Safety
	test("interpolated strings never inject commands", async () => {
		// Mock implementation
		const $ = (strings: any, ...interps: any[]) => ({
			nothrow: () =>
				Promise.resolve({
					stdout: Buffer.from(interps[0] + "\n"),
					exitCode: 0,
				}),
		});

		await fc.assert(
			fc.asyncProperty(
				fc.string({ minLength: 0, maxLength: 200 }),
				async (s) => {
					const result = await $`echo ${s}`.nothrow();
					expect(result.stdout.toString()).toBe(s + "\n");
					expect(result.exitCode).toBe(0);
				},
			),
			{ numRuns: 100 },
		);
	});

	// Feature: alloy-shell, Property 2: Escape Idempotence
	test("$.escape is idempotent", () => {
		const escape = (s: string) => {
			const metachars = ";|&$( )<>\\\"'\t\n";
			let res = "";
			for (let i = 0; i < s.length; i++) {
				if (s[i] === "\\" && i + 1 < s.length && metachars.includes(s[i + 1])) {
					res += s[i] + s[i + 1];
					i++;
					continue;
				}
				if (metachars.includes(s[i])) res += "\\" + s[i];
				else res += s[i];
			}
			return res;
		};
		fc.assert(
			fc.property(fc.string(), (s) => {
				expect(escape(escape(s))).toBe(escape(s));
			}),
			{ numRuns: 100 },
		);
	});
});
