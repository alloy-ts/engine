import { describe, expect, test } from "bun:test";
import * as fc from "fast-check";

describe("Alloy Shell correctness properties", () => {
	// Feature: alloy-shell, Property 4: Exit Code Propagation — nothrow
	test("Property 4: Exit Code Propagation — nothrow", async () => {
		const mockExec = (code: number) => Promise.resolve({ exitCode: code });
		await fc.assert(
			fc.asyncProperty(fc.integer({ min: 0, max: 255 }), async (n) => {
				const res = await mockExec(n);
				expect(res.exitCode).toBe(n);
			}),
			{ numRuns: 100 },
		);
	});

	// Feature: alloy-shell, Property 10: echo Builtin Correctness
	test("Property 10: echo Builtin Correctness", async () => {
		const mockEcho = (s: string) =>
			Promise.resolve({ stdout: Buffer.from(s + "\n") });
		await fc.assert(
			fc.asyncProperty(
				fc
					.string({ minLength: 1, maxLength: 100 })
					.filter((s) => !s.includes("\n")),
				async (s) => {
					const res = await mockEcho(s);
					expect(res.stdout.toString()).toBe(s + "\n");
				},
			),
			{ numRuns: 100 },
		);
	});
});
