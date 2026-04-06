// src/core/config/whitelist.test.ts
import { expect, test, describe } from "bun:test";
import { buildWhitelist } from "./whitelist";

describe("Whitelist", () => {
	test("Property 7: shellCapabilities Whitelist", () => {
		const capabilities = ["use-git", "use-curl"];
		const whitelist = buildWhitelist(capabilities);

		expect(whitelist.isAllowed("git")).toBe(true);
		expect(whitelist.isAllowed("curl")).toBe(true);
		expect(whitelist.isAllowed("alloy")).toBe(true);
		expect(whitelist.isAllowed("alloyx")).toBe(true);
		expect(whitelist.isAllowed("npm")).toBe(true);
		expect(whitelist.isAllowed("npx")).toBe(true);
		expect(whitelist.isAllowed("rm")).toBe(false);
	});
});
