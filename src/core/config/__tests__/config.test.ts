import { expect, test, describe, beforeAll, afterAll } from "bun:test";
import { interpolateEnv, findProjectRoot, ConfigLoader } from "../loader";
import { ConfigValidator } from "../validator";
import { PrettyPrinter } from "../pretty_printer";
import { writeFileSync, mkdirSync, rmSync, existsSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

describe("AlloyConfig", () => {
	test("environment interpolation", () => {
		process.env.TEST_VAR = "hello";
		expect(interpolateEnv("value-$TEST_VAR")).toBe("value-hello");
		expect(interpolateEnv("value-${TEST_VAR}")).toBe("value-hello");
		expect(interpolateEnv("value-$MISSING")).toBe("value-");
	});

	test("validator: target", () => {
		const validator = new ConfigValidator();
		// @ts-ignore
		expect(validator.validate({ target: "invalid" }).length).toBeGreaterThan(0);
		expect(validator.validate({ target: "local-first" }).length).toBe(0);
	});

	test("validator: useContainers + local-first", () => {
		const validator = new ConfigValidator();
		expect(validator.validate({ target: "local-first", useContainers: false }).length).toBeGreaterThan(0);
	});

    test("validator: run.shell + local-first", () => {
        const validator = new ConfigValidator();
        expect(validator.validate({ target: "local-first", run: { shell: "system" } }).length).toBeGreaterThan(0);
    });

    test("validator: test.seed warning", () => {
        const validator = new ConfigValidator();
        // Warning should not be an error
        expect(validator.validate({ test: { seed: 123, randomize: false } }).length).toBe(0);
    });

    test("PrettyPrinter round-trip", () => {
        const printer = new PrettyPrinter();
        const config = { telemetry: false, target: "server-first" as const };
        const toml = printer.print(config);
        expect(toml).toContain('telemetry = false');
        expect(toml).toContain('target = "server-first"');
    });
});
