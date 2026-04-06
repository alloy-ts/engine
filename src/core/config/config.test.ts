// src/core/config/config_test.test.ts
import { describe, it, expect } from "bun:test";
import { ConfigLoader } from "./loader";
import { ConfigValidator } from "./validator";
import { PrettyPrinter } from "./pretty_printer";
import * as fc from "fast-check";
import * as TOML from "@iarna/toml";
import { existsSync, writeFileSync, unlinkSync } from "node:fs";

describe("ConfigLoader", () => {
	it("should parse a valid TOML file", () => {
		const loader = new ConfigLoader();
		const toml = `
target = "server-first"
logLevel = "debug"
[test]
coverage = true
[define]
API = "http://localhost"
`;
		writeFileSync("test_config.toml", toml);
		const result = loader.parse("test_config.toml");
		expect(result.raw.target).toBe("server-first");
		expect(result.raw.logLevel).toBe("debug");
		expect((result.raw as any).test.coverage).toBe(true);
		expect((result.raw as any).define.API).toBe("http://localhost");
		unlinkSync("test_config.toml");
	});

	it("should interpolate environment variables", () => {
		const loader = new ConfigLoader();
		process.env.TEST_VAR = "FOO";
		const raw = {
			registry: "http://$TEST_VAR.com",
			nested: {
				url: "http://${TEST_VAR}.com",
			},
		};
		const result = loader.interpolate(raw);
		expect(result.registry).toBe("http://FOO.com");
		expect(result.nested.url).toBe("http://FOO.com");
		delete process.env.TEST_VAR;
	});

	it("should merge with precedence CLI > Workspace > User > Defaults", () => {
		const loader = new ConfigLoader();
		const user = { logLevel: "error", target: "local-first" };
		const workspace = { logLevel: "warn", test: { root: "./tests" } };
		const cli = { logLevel: "debug" };

		const merged = loader.merge(user, workspace, cli as any);
		expect(merged.logLevel).toBe("debug"); // CLI wins
		expect((merged as any).test.root).toBe("./tests"); // Workspace wins
		expect(merged.target).toBe("local-first"); // User wins
	});

	it("should merge [define] at key level", () => {
		const loader = new ConfigLoader();
		const user = { define: { A: "1", B: "2" } };
		const workspace = { define: { B: "3", C: "4" } };
		const merged = loader.merge(user, workspace);
		expect((merged as any).define).toEqual({ A: "1", B: "3", C: "4" });
	});
});

describe("ConfigValidator", () => {
	it("should warn on unknown keys", () => {
		const validator = new ConfigValidator();
		const raw = { unknown_top: 1, test: { unknown_nested: 2 } };
		const result = validator.validate(raw, "test.toml");
		expect(result.warnings.some((w) => w.fieldPath === "unknown_top")).toBe(true);
		expect(result.warnings.some((w) => w.fieldPath === "test.unknown_nested")).toBe(
			true,
		);
	});

	it("should validate cross-field rule: run.shell requires server-first", () => {
		const validator = new ConfigValidator();
		const raw = { target: "local-first", run: { shell: "system" } };
		const result = validator.validate(raw, "test.toml");
		expect(result.valid).toBe(false);
		expect(result.errors.some((e) => e.fieldPath === "run.shell")).toBe(true);
	});
});

describe("Property Tests", () => {
	it("Property 1: Round-Trip Serialization", () => {
		const printer = new PrettyPrinter();
		const loader = new ConfigLoader();

		fc.assert(
			fc.property(
				fc.record({
					target: fc.constantFrom("local-first", "server-first"),
					logLevel: fc.constantFrom("debug", "warn", "error"),
					cryptomem: fc.boolean(),
					test: fc.record(
						{
							root: fc.string(),
							coverage: fc.boolean(),
						},
						{ requiredKeys: [] },
					),
				}),
				(config: any) => {
					const tomlStr = printer.print(config);
					const parsed = loader.parseString(tomlStr).raw;
					// Deep cleaning to match JSON round-trip
					const expected = JSON.parse(JSON.stringify(config));
					expect(parsed).toEqual(expected);
				},
			),
			{ numRuns: 100 },
		);
	});
});
