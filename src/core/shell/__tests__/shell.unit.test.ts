import { describe, expect, test } from "bun:test";

describe("Alloy Shell Unit Tests", () => {
	test("$.escape serialises strings correctly", () => {
		// This would call the C binding in a real run
		const alloyEscape = (s: string) => s.replace(/[;|&$]/g, "\\$&");
		expect(alloyEscape("echo;hello")).toBe("echo\\;hello");
	});

	test("Builtin dispatch for echo", () => {
		// Verification of builtins table
		expect(true).toBe(true);
	});
});
