// src/core/config/loader.test.ts
import { expect, test, describe } from "bun:test";
import * as fc from "fast-check";
import { ConfigLoader } from "./loader";
import { AlloyConfig, BUILTIN_DEFAULTS } from "./config";

describe("ConfigLoader", () => {
	const loader = new ConfigLoader();

	test("Property 2: Merge Precedence", () => {
		// Feature: alloyconfig-toml, Property 2: CLI > LocalConfig > GlobalConfig > defaults
		const configArb = fc.record(
			{
				target: fc.oneof(fc.constant("local-first"), fc.constant("server-first")),
				logLevel: fc.oneof(
					fc.constant("debug"),
					fc.constant("warn"),
					fc.constant("error"),
				),
				telemetry: fc.boolean(),
				test: fc.record({
					root: fc.string(),
					coverage: fc.boolean(),
				}),
				define: fc.dictionary(fc.string(), fc.string()),
			},
			{ requiredKeys: [] },
		);

		fc.assert(
			fc.property(configArb, configArb, configArb, (global, local, cli) => {
				const merged = loader.merge(global, local, cli);

				const keys: (keyof AlloyConfig)[] = ["target", "logLevel", "telemetry", "test"];
				for (const key of keys) {
					if (cli[key] !== undefined) {
						expect(merged[key]).toEqual(cli[key] as any);
					} else if (local[key] !== undefined) {
						expect(merged[key]).toEqual(local[key] as any);
					} else if (global[key] !== undefined) {
						expect(merged[key]).toEqual(global[key] as any);
					} else {
						expect(merged[key]).toEqual(BUILTIN_DEFAULTS[key] as any);
					}
				}

				if (global.define || local.define || cli.define) {
					const expectedDefine = {
						...BUILTIN_DEFAULTS.define,
						...global.define,
						...local.define,
						...cli.define,
					};
					expect(merged.define).toEqual(expectedDefine);
				}
			}),
			{ numRuns: 100 },
		);
	});

	test("Property 3: Env Var Interpolation", () => {
		// Feature: alloyconfig-toml, Property 3: string fields with $VAR are substituted
		fc.assert(
			fc.property(
				fc.string({
					minLength: 1,
					alphabet: "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_",
				}),
				fc.string({
					minLength: 1,
					alphabet: "abcdefghijklmnopqrstuvwxyz0123456789-",
				}),
				(varNameFragment, varValue) => {
					// Manually clean varNameFragment to ensure no weird chars
					const varName = "TEST_VAR_" + varNameFragment.replace(/[^A-Z0-9_]/g, "X");
					process.env[varName] = varValue;
					const raw: any = { install: { registry: `https://registry.example.com/$${varName}/` } };
					const interpolated = loader.interpolate(raw);
					expect(interpolated.install.registry).toContain(varValue);
					delete process.env[varName];
				},
			),
			{ numRuns: 100 },
		);
	});
});
