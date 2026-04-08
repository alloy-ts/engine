import { expect, test, describe } from "bun:test";
import { PluginLoader } from "./loader";
import { AlloyConfig } from "../config/config";
import { writeFileSync, mkdirSync, rmdirSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

describe("PluginLoader", () => {
	const loader = new PluginLoader();

	test("Topological Sort", () => {
		const config: AlloyConfig = {
			plugins: [
				{ name: "A", path: "./A", dependencies: ["B"] },
				{ name: "B", path: "./B", dependencies: [] },
				{ name: "C", path: "./C", dependencies: ["A"] },
			]
		};

		// Mock existence of manifest files
		const setup = (name: string, deps: string[]) => {
			const dir = join(tmpdir(), name);
			try { mkdirSync(dir); } catch(e) {}
			writeFileSync(join(dir, "alloy-plugin.json"), JSON.stringify({
				name, version: "1.0.0", entry: "index.js", dependencies: deps
			}));
			return dir;
		};

		const dirA = setup("A", ["B"]);
		const dirB = setup("B", []);
		const dirC = setup("C", ["A"]);

		const configWithPaths: AlloyConfig = {
			plugins: [
				{ name: "A", path: dirA, dependencies: ["B"] },
				{ name: "B", path: dirB, dependencies: [] },
				{ name: "C", path: dirC, dependencies: ["A"] },
			]
		};

		const resolved = loader.resolvePlugins(configWithPaths);
		expect(resolved.map(n => n.name)).toEqual(["B", "A", "C"]);
	});
});
