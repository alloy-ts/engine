import { AlloyConfig, PluginEntry, PluginManifest } from "./config";
import { PluginManifestSchema } from "./schema";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import * as TOML from "@iarna/toml";

/**
 * Handles plugin resolution, dependency sorting, and manifest validation.
 * Requirement 17.1-17.8.
 */
export class PluginRegistry {
	private plugins = new Map<string, { entry: PluginEntry; manifest: PluginManifest }>();
	private loadOrder: string[] = [];

	constructor(private config: AlloyConfig) {}

	async resolveAndValidate() {
		const entries = this.config.plugins || [];

		// 1. Resolve and Load Manifests
		for (const entry of entries) {
			const resolved = this.resolveEntry(entry);
			if (resolved.entry.disabled) continue;

			const manifest = this.loadManifest(resolved.path);
			this.validateManifest(manifest);
			this.plugins.set(manifest.name, { entry: resolved.entry, manifest });
		}

		// 2. Topological Sort (Kahn's Algorithm)
		this.topoSort();
	}

	private resolveEntry(entry: string | PluginEntry): {
		entry: PluginEntry;
		path: string;
	} {
		if (typeof entry === "string") {
			const pluginEntry: PluginEntry = { path: entry };
			// If it's a relative path, use it directly, otherwise it might be an npm package
			return { entry: pluginEntry, path: entry };
		}
		return { entry, path: entry.path || "" };
	}

	private loadManifest(path: string): PluginManifest {
		const manifestPaths = [
			join(path, "alloy-plugin.toml"),
			join(path, "alloy-plugin.json"),
		];

		for (const p of manifestPaths) {
			if (existsSync(p)) {
				const content = readFileSync(p, "utf-8");
				if (p.endsWith(".toml")) {
					return TOML.parse(content) as unknown as PluginManifest;
				}
				return JSON.parse(content) as unknown as PluginManifest;
			}
		}
		throw new Error(`Plugin manifest not found in ${path}`);
	}

	private validateManifest(manifest: PluginManifest) {
		const result = PluginManifestSchema.safeParse(manifest);
		if (!result.success) {
			throw new Error(
				`Invalid plugin manifest for ${manifest.name}: ${result.error.message}`,
			);
		}
	}

	private topoSort() {
		const adj = new Map<string, string[]>();
		const inDegree = new Map<string, number>();

		for (const [name, { manifest }] of this.plugins) {
			inDegree.set(name, 0);
			adj.set(name, []);
		}

		for (const [name, { manifest }] of this.plugins) {
			const deps = manifest.dependencies || [];
			for (const dep of deps) {
				if (this.plugins.has(dep)) {
					adj.get(dep)!.push(name);
					inDegree.set(name, (inDegree.get(name) || 0) + 1);
				}
			}
		}

		const queue = [];
		for (const [name, degree] of inDegree) {
			if (degree === 0) queue.push(name);
		}

		while (queue.length > 0) {
			const u = queue.shift()!;
			this.loadOrder.push(u);
			for (const v of adj.get(u) || []) {
				inDegree.set(v, inDegree.get(v)! - 1);
				if (inDegree.get(v) === 0) queue.push(v);
			}
		}

		if (this.loadOrder.length !== this.plugins.size) {
			throw new Error("Circular plugin dependencies detected");
		}
	}

	getLoadOrder() {
		return this.loadOrder.map((name) => this.plugins.get(name)!);
	}
}
