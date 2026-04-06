// src/core/config/validator.test.ts
import { expect, test, describe } from "bun:test";
import * as fc from "fast-check";
import { ConfigValidator } from "./validator";

describe("ConfigValidator", () => {
	const validator = new ConfigValidator();

	test("Property 4: Validation Completeness (Type Errors)", () => {
		// Feature: alloyconfig-toml, Property 4: validator reports exactly N errors for N type violations
		// We'll test with a few specific type errors
		const raw: any = {
			target: 123, // should be string
			logLevel: true, // should be string
			telemetry: "yes", // should be boolean
		};
		const result = validator.validate(raw, "alloyconfig.toml");
		expect(result.errors.length).toBe(3);
		expect(result.valid).toBe(false);
	});

	test("Property 5: Unknown Key Warning", () => {
		// Feature: alloyconfig-toml, Property 5: K unknown keys produce exactly K warnings
		const raw: any = {
			unknownTopLevel: 1,
			test: {
				unknownInTest: 2,
			},
		};
		const result = validator.validate(raw, "alloyconfig.toml");
		expect(result.warnings.filter((w) => w.kind === "unknown_key").length).toBe(2);
	});

	test("Cross-field validation: run.shell requires server-first", () => {
		const raw: any = {
			target: "local-first",
			run: { shell: "system" },
		};
		const result = validator.validate(raw, "alloyconfig.toml");
		expect(result.errors.some((e) => e.kind === "cross_field_error")).toBe(true);
	});
});
