import { test, expect } from "bun:test";
import { ConfigValidator } from "./validator";

test("ConfigValidator - unknown key warning", () => {
	const validator = new ConfigValidator();
	const raw = {
		test: {
			unknownField: "foo",
		},
	};
	const result = validator.validate(raw as any, "alloyconfig.toml");
	expect(result.warnings.length).toBe(1);
	expect(result.warnings[0].fieldPath).toBe("test.unknownField");
});

test("ConfigValidator - cross-field rules", () => {
	const validator = new ConfigValidator();

	// Case 1: test.seed without test.randomize
	const raw1 = {
		test: {
			seed: 123,
			randomize: false,
		},
	};
	const result1 = validator.validate(raw1 as any, "alloyconfig.toml");
	expect(result1.warnings.some(w => w.fieldPath === "test.seed")).toBe(true);

	// Case 2: run.shell = "system" with target = "local-first"
	const raw2 = {
		target: "local-first",
		run: {
			shell: "system",
		},
	};
	const result2 = validator.validate(raw2 as any, "alloyconfig.toml");
	expect(result2.errors.some(e => e.fieldPath === "run.shell")).toBe(true);

	// Case 3: useContainers = false with target = "local-first"
	const raw3 = {
		target: "local-first",
		useContainers: false,
	};
	const result3 = validator.validate(raw3 as any, "alloyconfig.toml");
	expect(result3.errors.some(e => e.fieldPath === "useContainers")).toBe(true);

	// Case 4: shellCapabilityImages entry not in shellCapabilities
	const raw4 = {
		shellCapabilities: ["ls"],
		shellCapabilityImages: {
			cat: "alpine:latest",
		},
	};
	const result4 = validator.validate(raw4 as any, "alloyconfig.toml");
	expect(result4.errors.some(e => e.fieldPath === "shellCapabilityImages.cat")).toBe(true);
});
