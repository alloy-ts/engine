// src/core/config/plugins.test.ts
import { expect, test, describe } from "bun:test";
import { ResolvedPlugin, sortPlugins } from "./plugins";

describe("Plugin Loading", () => {
	test("Property 10: alloy:config is always first (implicit in topo sort if dependants exist)", () => {
		const plugins: ResolvedPlugin[] = [
			{
				name: "my-plugin",
				manifest: { name: "my-plugin", version: "1.0.0", entry: "./index.js", dependencies: ["alloy:config"] },
				config: {},
				path: "/plugins/my-plugin",
			},
			{
				name: "alloy:config",
				manifest: { name: "alloy:config", version: "1.0.0", entry: "./index.js" },
				config: {},
				path: "/plugins/config",
			},
		];

		const sorted = sortPlugins(plugins);
		expect(sorted[0].name).toBe("alloy:config");
		expect(sorted[1].name).toBe("my-plugin");
	});

	test("Circular dependency detection", () => {
		const plugins: ResolvedPlugin[] = [
			{
				name: "plugin-a",
				manifest: { name: "plugin-a", version: "1.0.0", entry: "./index.js", dependencies: ["plugin-b"] },
				config: {},
				path: "/plugins/a",
			},
			{
				name: "plugin-b",
				manifest: { name: "plugin-b", version: "1.0.0", entry: "./index.js", dependencies: ["plugin-a"] },
				config: {},
				path: "/plugins/b",
			},
		];

		expect(() => sortPlugins(plugins)).toThrow("Circular dependency detected");
	});
});
