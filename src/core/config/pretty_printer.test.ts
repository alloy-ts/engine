// src/core/config/pretty_printer.test.ts
import { expect, test, describe } from "bun:test";
import { PrettyPrinter } from "./pretty_printer";
import * as TOML from "@iarna/toml";
import type { AlloyConfig } from "./config";

describe("PrettyPrinter", () => {
	const printer = new PrettyPrinter();

	test("Property 1: Round-Trip Serialization", () => {
		const config: AlloyConfig = {
			target: "server-first",
			logLevel: "debug",
			telemetry: false,
			define: {
				API_URL: "https://api.example.com",
			},
			test: {
				root: "./tests",
				coverage: true,
			},
		};

		const toml = printer.print(config);
		const parsed = TOML.parse(toml) as any;

		// Deep comparison of the parts we care about
		expect(parsed.target).toBe(config.target);
		expect(parsed.logLevel).toBe(config.logLevel);
		expect(parsed.telemetry).toBe(config.telemetry);
		expect(parsed.define.API_URL).toBe(config.define!.API_URL);
		expect(parsed.test.root).toBe(config.test!.root);
		expect(parsed.test.coverage).toBe(config.test!.coverage);
	});
});
