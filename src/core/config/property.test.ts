import { test, expect } from "bun:test";
import { ConfigLoader } from "./loader";
import { ConfigValidator } from "./validator";
import * as fc from "fast-check";

test("Property 2: Merge Precedence", () => {
	// Feature: alloyconfig-toml, Property 2: Merge Precedence
	const loader = new ConfigLoader();

	fc.assert(
		fc.property(
			fc.record({ logLevel: fc.constantFrom("debug", "warn", "error") }),
			fc.record({ logLevel: fc.constantFrom("debug", "warn", "error") }),
			fc.record({ logLevel: fc.constantFrom("debug", "warn", "error") }),
			(user, workspace, cli) => {
				const merged = (loader as any).merge(user, workspace, cli);
				expect(merged.logLevel).toBe(cli.logLevel);
			},
		),
	);
});

test("Property 3: Env Var Interpolation", () => {
	// Feature: alloyconfig-toml, Property 3: Env Var Interpolation
	const loader = new ConfigLoader();

	fc.assert(
		fc.property(fc.string(), fc.string(), (varName, varValue) => {
			// Ensure varName is a valid env var name
			const safeVarName = "VAR_" + varName.replace(/[^a-zA-Z0-9_]/g, "");
			if (safeVarName === "VAR_") return;

			process.env[safeVarName] = varValue;
			const config = { field: `$${safeVarName}` };
			const interpolated = (loader as any).interpolate(config);
			expect(interpolated.field).toBe(varValue);
		}),
	);
});

test("Property 4: Validation Completeness", () => {
	// Feature: alloyconfig-toml, Property 4: Validation Completeness
	const validator = new ConfigValidator();

	fc.assert(
		fc.property(
			fc.record({
				target: fc.constantFrom("local-first", "server-first"),
				logLevel: fc.constantFrom("debug", "warn", "error"),
			}),
			(config) => {
				const result = validator.validate(config as any, "test.toml");
				expect(result.valid).toBe(true);
			},
		),
	);
});
