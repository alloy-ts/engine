import {
	AlloyConfig,
	PluginEntry,
	PluginManifest,
	PluginConfigSchema,
} from "../config/config";
import { ConfigValidator } from "../config/validator";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { PluginManifestSchema } from "../config/schema";
import * as TOML from "@iarna/toml";

export interface PluginNode {
	name: string;
	path: string;
	manifest: PluginManifest;
	dependencies: string[];
	config?: Record<string, unknown>;
}

export class PluginLoader {
	private validator = new ConfigValidator();

	resolvePlugins(config: AlloyConfig): PluginNode[] {
		const plugins = config.plugins || [];
		const resolved: PluginNode[] = [];

		for (const entry of plugins) {
			const node = this.resolveEntry(entry);
			if (node) resolved.push(node);
		}

		return this.topologicalSort(resolved);
	}

	private resolveEntry(entry: string | PluginEntry): PluginNode | null {
		const path = typeof entry === "string" ? entry : entry.path;
		if (!path) return null;

		const manifestPath = existsSync(join(path, "alloy-plugin.toml"))
			? join(path, "alloy-plugin.toml")
			: join(path, "alloy-plugin.json");

		if (!existsSync(manifestPath)) {
			console.error(`error: manifest not found at ${path}`);
			return null;
		}

		// Parse manifest (JSON or TOML)
		let manifest: PluginManifest;
		const content = readFileSync(manifestPath, "utf-8");
		if (manifestPath.endsWith(".json")) {
			manifest = JSON.parse(content);
		} else {
			manifest = TOML.parse(content) as unknown as PluginManifest;
		}

		// Validate manifest
		const result = PluginManifestSchema.safeParse(manifest);
		if (!result.success) {
			console.error(`error: invalid manifest at ${manifestPath}`, result.error);
			return null;
		}

		return {
			name: manifest.name,
			path: path,
			manifest: manifest,
			dependencies: manifest.dependencies || [],
			config: typeof entry === "string" ? {} : entry.config,
		};
	}

	private topologicalSort(nodes: PluginNode[]): PluginNode[] {
		const sorted: PluginNode[] = [];
		const visited = new Set<string>();
		const visiting = new Set<string>();

		const visit = (node: PluginNode) => {
			if (visiting.has(node.name)) {
				throw new Error(`Circular dependency detected: ${node.name}`);
			}
			if (!visited.has(node.name)) {
				visiting.add(node.name);
				for (const depName of node.dependencies) {
					const depNode = nodes.find((n) => n.name === depName);
					if (depNode) visit(depNode);
				}
				visiting.delete(node.name);
				visited.add(node.name);
				sorted.push(node);
			}
		};

		for (const node of nodes) {
			visit(node);
		}

		return sorted;
	}
}
