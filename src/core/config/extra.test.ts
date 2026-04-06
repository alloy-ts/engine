import { describe, it, expect } from "bun:test";
import { Whitelist, buildWhitelist } from "./whitelist";
import { ContainerManager } from "./container";
import { PluginRegistry } from "./plugins";
import { writeFileSync, mkdirSync, unlinkSync, rmdirSync } from "node:fs";
import { join } from "node:path";

describe("Whitelist", () => {
	it("should correctly whitelist binaries from shellCapabilities", () => {
		const whitelist = buildWhitelist(["use-git", "use-curl"]);
		expect(whitelist.isAllowed("git")).toBe(true);
		expect(whitelist.isAllowed("curl")).toBe(true);
		expect(whitelist.isAllowed("npm")).toBe(true); // auto-whitelisted
		expect(whitelist.isAllowed("npx")).toBe(true); // auto-whitelisted
		expect(whitelist.isAllowed("rm")).toBe(false);
	});

	it("should whitelist binaries from package.json", () => {
		const pkg = {
			dependencies: { "my-tool": "1.0.0" },
		};
		const whitelist = buildWhitelist([], pkg);
		expect(whitelist.isAllowed("my-tool")).toBe(true);
	});
});

describe("ContainerManager", () => {
	it("should simulate running a command in a container", async () => {
		const manager = new ContainerManager();
		const result = await manager.runInContainer("ls", ["-la"], {
			capability: "use-git",
		});
		expect(result.exitCode).toBe(0);
		expect(result.stdout).toBeDefined();
	});
});

describe("PluginRegistry", () => {
	it("should topologically sort plugins based on dependencies", async () => {
		// Mock config and manifests
		const config = {
			plugins: [{ path: "./plugin-a" }, { path: "./plugin-b" }],
		};

		mkdirSync("./plugin-a", { recursive: true });
		mkdirSync("./plugin-b", { recursive: true });

		const manifestA = {
			name: "plugin-a",
			version: "1.0.0",
			entry: "./index.js",
			dependencies: ["plugin-b"],
		};
		const manifestB = {
			name: "plugin-b",
			version: "1.0.0",
			entry: "./index.js",
		};

		writeFileSync("./plugin-a/alloy-plugin.json", JSON.stringify(manifestA));
		writeFileSync("./plugin-b/alloy-plugin.json", JSON.stringify(manifestB));

		const registry = new PluginRegistry(config as any);
		await registry.resolveAndValidate();
		const loadOrder = registry.getLoadOrder().map((p) => p.manifest.name);

		expect(loadOrder).toEqual(["plugin-b", "plugin-a"]);

		unlinkSync("./plugin-a/alloy-plugin.json");
		unlinkSync("./plugin-b/alloy-plugin.json");
		rmdirSync("./plugin-a");
		rmdirSync("./plugin-b");
	});
});
