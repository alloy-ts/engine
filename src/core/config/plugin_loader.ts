// src/core/config/plugin_loader.ts
import { PluginEntry, PluginManifest } from "./config";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import * as TOML from "@iarna/toml";

export interface LoadedPlugin {
	name: string;
	manifest: PluginManifest;
	config: any;
}

export function sortPlugins(plugins: LoadedPlugin[]): LoadedPlugin[] {
	const result: LoadedPlugin[] = [];
	const visited = new Set<string>();
	const stack = new Set<string>();

	function visit(plugin: LoadedPlugin) {
		if (stack.has(plugin.name)) {
			throw new Error(`Circular dependency detected: ${plugin.name}`);
		}
		if (visited.has(plugin.name)) return;

		stack.add(plugin.name);
		const deps = plugin.manifest.dependencies || [];
		for (const depName of deps) {
			const dep = plugins.find((p) => p.name === depName);
			if (dep) {
				visit(dep);
			} else {
				throw new Error(`Missing dependency: ${depName} for plugin ${plugin.name}`);
			}
		}
		stack.delete(plugin.name);
		visited.add(plugin.name);
		result.push(plugin);
	}

	for (const plugin of plugins) {
		visit(plugin);
	}

	return result;
}

export function loadPluginManifest(path: string): PluginManifest {
	let manifestPath = join(path, "alloy-plugin.toml");
	if (!existsSync(manifestPath)) {
		manifestPath = join(path, "alloy-plugin.json");
	}

	if (!existsSync(manifestPath)) {
		throw new Error(`Plugin manifest not found in ${path}`);
	}

	const content = readFileSync(manifestPath, "utf-8");
	if (manifestPath.endsWith(".json")) {
		return JSON.parse(content);
	} else {
		return TOML.parse(content) as any;
	}
}
