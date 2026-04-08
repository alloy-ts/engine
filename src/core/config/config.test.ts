import { expect, test, describe, beforeAll, afterAll } from "bun:test";
import { ConfigLoader } from "./loader";
import { ConfigValidator } from "./validator";
import { PrettyPrinter } from "./pretty_printer";
import { buildWhitelist } from "./whitelist";
import { randomizeOrder } from "../test/randomizer";
import * as fc from "fast-check";
import { writeFileSync, unlinkSync, mkdirSync, rmdirSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

describe("AlloyConfig System", () => {
	const loader = new ConfigLoader();
	const validator = new ConfigValidator();
	const printer = new PrettyPrinter();

	test("Property 1: Round-Trip Serialization", () => {
		// Feature: alloyconfig-toml, Property 1: parse(prettyPrint(config)) deep-equals config
		const arbitraryConfig = fc.record({
			target: fc.constantFrom("local-first", "server-first"),
			logLevel: fc.constantFrom("debug", "warn", "error"),
			telemetry: fc.boolean(),
			test: fc.record({
				root: fc.string(),
				coverage: fc.boolean(),
			}, { requiredKeys: [] }),
		});

		fc.assert(
			fc.property(arbitraryConfig, (config) => {
				const toml = printer.print(config as any);
				// printer.print might add defaults if not careful, but our implementation just stringifies
				// We need a way to parse TOML string back to object.
				// Since we don't want to write to file, we can use TOML.parse directly if exported or via loader.parse with a temp file.
				const tmpFile = join(tmpdir(), `alloytest-${Math.random()}.toml`);
				writeFileSync(tmpFile, toml);
				const parsed = loader.parse(tmpFile).raw;
				unlinkSync(tmpFile);

				// Deep equality check (ignoring undefineds)
				expect(JSON.parse(JSON.stringify(parsed))).toEqual(JSON.parse(JSON.stringify(config)));
			}),
			{ numRuns: 100 },
		);
	});

	test("Property 2: Merge Precedence", () => {
		// Feature: alloyconfig-toml, Property 2: CLI > LocalConfig > GlobalConfig > defaults
		fc.assert(
			fc.property(
				fc.record({ logLevel: fc.constantFrom("debug", "warn", "error") }, { requiredKeys: [] }),
				fc.record({ logLevel: fc.constantFrom("debug", "warn", "error") }, { requiredKeys: [] }),
				fc.record({ logLevel: fc.constantFrom("debug", "warn", "error") }, { requiredKeys: [] }),
				(global, local, cli) => {
					const merged = loader.merge(global as any, local as any, cli as any);
					if (cli.logLevel) expect(merged.logLevel).toBe(cli.logLevel);
					else if (local.logLevel) expect(merged.logLevel).toBe(local.logLevel);
					else if (global.logLevel) expect(merged.logLevel).toBe(global.logLevel);
					else expect(merged.logLevel).toBe("warn"); // Default
				}
			)
		);
	});

	test("Property 3: Env Var Interpolation", () => {
		// Feature: alloyconfig-toml, Property 3: string fields with $VAR are substituted
		fc.assert(
			fc.property(fc.string({ minLength: 1, maxLength: 10 }), fc.string(), (varName, varValue) => {
				const envName = `TEST_${varName.toUpperCase().replace(/[^A-Z0-9]/g, "_")}`;
				process.env[envName] = varValue;
				const config = { debug: { editor: `$${envName}/world` } };
				const interpolated = loader.interpolate(config);
				expect(interpolated.debug?.editor).toBe(`${varValue}/world`);
				delete process.env[envName];
			})
		);
	});

	test("Property 7: shellCapabilities Whitelist", () => {
		// Feature: alloyconfig-toml, Property 7: only whitelisted binaries are permitted
		const whitelist = buildWhitelist(["use-git", "curl"]);
		expect(whitelist.isAllowed("git")).toBe(true);
		expect(whitelist.isAllowed("curl")).toBe(true);
		expect(whitelist.isAllowed("npm")).toBe(true);
		expect(whitelist.isAllowed("rm")).toBe(false);
	});

	test("Validation: Cross-field rules", () => {
		const config = {
			target: "local-first" as const,
			run: { shell: "system" as const }
		};
		const result = validator.validate(config as any, "alloyconfig.toml");
		expect(result.valid).toBe(false);
		expect(result.errors[0].message).toBe('value "system" requires target = "server-first"');
	});

	test("Property 9: Seed Determinism", () => {
		const items = ["a", "b", "c", "d", "e"];
		const seed = 12345;
		const order1 = randomizeOrder(items, seed);
		const order2 = randomizeOrder(items, seed);
		expect(order1).toEqual(order2);

		const order3 = randomizeOrder(items, 54321);
		expect(order1).not.toEqual(order3);
	});

	test("Property 13: Unknown Plugin Config Key Warning", () => {
		const pluginConfigs = {
			"test-plugin": {
				known: "value",
				unknown1: "v1",
				unknown2: "v2"
			}
		};
		const registeredSchemas = new Map([
			["test-plugin", { configs: { known: { default: "def" } } }]
		]);

		const result = validator.validatePluginConfigs(pluginConfigs, registeredSchemas as any, "alloyconfig.toml");
		expect(result.warnings.length).toBe(2);
		expect(result.warnings[0].fieldPath).toBe("plugins.test-plugin.unknown1");
	});
});
