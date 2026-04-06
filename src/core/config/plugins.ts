// src/core/config/plugins.ts
import { AlloyConfig, PluginEntry, PluginManifest } from "./config";
import { existsSync, readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import * as TOML from "@iarna/toml";

export interface ResolvedPlugin {
	name: string;
	manifest: PluginManifest;
	config: Record<string, unknown>;
	path: string;
}

export function resolvePlugin(
	entry: string | PluginEntry,
	cwd: string = process.cwd(),
): ResolvedPlugin {
	let pluginName: string | undefined;
	let manifestPath: string;
	let inlineConfig: Record<string, unknown> = {};

	if (typeof entry === "string") {
		// entry is path or package name
		if (entry.startsWith(".") || entry.startsWith("/")) {
			manifestPath = entry.endsWith(".toml") || entry.endsWith(".json")
				? (entry.startsWith(".") ? join(cwd, entry) : entry)
				: join(cwd, entry, "alloy-plugin.toml");
		} else {
			manifestPath = join(cwd, "node_modules", entry, "alloy-plugin.toml");
		}
	} else {
		pluginName = entry.name;
		if (entry.path) {
			manifestPath = entry.path.startsWith(".") || entry.path.startsWith("/")
				? (entry.path.startsWith(".") ? join(cwd, entry.path) : entry.path)
				: entry.path;
			if (!manifestPath.endsWith(".toml") && !manifestPath.endsWith(".json")) {
				manifestPath = join(manifestPath, "alloy-plugin.toml");
			}
		} else if (pluginName) {
			manifestPath = join(cwd, "node_modules", pluginName, "alloy-plugin.toml");
		} else {
			throw new Error("Plugin entry must have either a name or a path");
		}
		inlineConfig = entry.config || {};
	}

	// Requirement 18.1: Read and parse the manifest
	let manifestRaw: any;
	if (existsSync(manifestPath)) {
		const content = readFileSync(manifestPath, "utf-8");
		manifestRaw = manifestPath.endsWith(".json")
			? JSON.parse(content)
			: TOML.parse(content);
	} else {
		// Try fallback to .json if .toml not found
		const jsonPath = manifestPath.replace(/\.toml$/, ".json");
		if (existsSync(jsonPath)) {
			manifestRaw = JSON.parse(readFileSync(jsonPath, "utf-8"));
		} else {
			throw new Error(`Plugin manifest not found at ${manifestPath}`);
		}
	}

	const manifest: PluginManifest = {
		name: manifestRaw.name,
		version: manifestRaw.version,
		description: manifestRaw.description,
		entry: manifestRaw.entry,
		dependencies: manifestRaw.dependencies,
		ffi: manifestRaw.ffi,
		capabilities: manifestRaw.capabilities,
	};

	// Requirement 18.2: Validation logic
	if (!manifest.name || !manifest.version || !manifest.entry) {
		throw new Error(`Plugin manifest at ${manifestPath} is missing required fields (name, version, entry)`);
	}

	return {
		name: manifest.name,
		manifest,
		config: inlineConfig,
		path: dirname(manifestPath),
	};
}

/**
 * Kahn's algorithm for topological sort (Requirement 17.4)
 */
export function sortPlugins(plugins: ResolvedPlugin[]): ResolvedPlugin[] {
	const result: ResolvedPlugin[] = [];
	const inDegree = new Map<string, number>();
	const graph = new Map<string, string[]>();
	const nameToPlugin = new Map<string, ResolvedPlugin>();

	for (const p of plugins) {
		nameToPlugin.set(p.name, p);
		inDegree.set(p.name, 0);
		graph.set(p.name, []);
	}

	for (const p of plugins) {
		const deps = p.manifest.dependencies || [];
		for (const dep of deps) {
			if (!graph.has(dep)) continue; // ignore missing optional deps
			graph.get(dep)!.push(p.name);
			inDegree.set(p.name, (inDegree.get(p.name) || 0) + 1);
		}
	}

	const queue: string[] = [];
	for (const [name, degree] of inDegree.entries()) {
		if (degree === 0) queue.push(name);
	}

	while (queue.length > 0) {
		const u = queue.shift()!;
		result.push(nameToPlugin.get(u)!);

		for (const v of graph.get(u) || []) {
			inDegree.set(v, inDegree.get(v)! - 1);
			if (inDegree.get(v) === 0) queue.push(v);
		}
	}

	if (result.length !== plugins.length) {
		throw new Error("Circular dependency detected in plugins");
	}

	return result;
}
