import { test, expect } from "bun:test";
import { PrettyPrinter } from "./pretty_printer";
import { ConfigLoader } from "./loader";
import { AlloyConfig } from "./config";
import * as TOML from "@iarna/toml";

test("PrettyPrinter - Round-trip", () => {
	const printer = new PrettyPrinter();
	const config: AlloyConfig = {
		target: "local-first",
		logLevel: "debug",
		define: { A: "1", B: "2" },
		test: {
			root: "tests",
			coverage: true,
		},
	};

	const printed = printer.print(config);
	const parsed = TOML.parse(printed);

	// Compare objects, ignoring potential subtle diffs in empty sections if any
	expect(parsed.target).toBe(config.target);
	expect(parsed.logLevel).toBe(config.logLevel);
	expect((parsed.define as any).A).toBe(config.define?.A);
	expect((parsed.test as any).root).toBe(config.test?.root);
});
