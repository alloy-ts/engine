// tests/build.test.ts
import { expect, test, describe } from "bun:test";
import * as fc from "fast-check";
import { build } from "../src/core/build";

// Feature: alloyscript-dual-engine, Property 27: Alloy.build entry-point coverage
// Feature: alloyscript-dual-engine, Property 28: Alloy.build minified output is no larger than unminified
// Feature: alloyscript-dual-engine, Property 29: Alloy.build sourcemap artifact presence
// Feature: alloyscript-dual-engine, Property 30: Alloy.build error collection — never throws

describe("build", () => {
	test("Property 30: Alloy.build error collection — never throws", async () => {
		const result = await build({ entrypoints: ["non-existent.ts"] });
		expect(result.success).toBe(false);
		expect(result.logs.length).toBeGreaterThan(0);
	});
});
