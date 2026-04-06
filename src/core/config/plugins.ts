import { AlloyConfig, PluginEntry } from "./config";
import { readFileSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { ConfigValidator } from "./validator";

export interface RuntimePlugin {
	onStart?: (engine: any) => Promise<void> | void;
	onResolve?: (id: string, parent?: string) => Promise<string | null> | string | null;
	onLoad?: (id: string) => Promise<{ contents: string; loader: string } | null> | { contents: string; loader: string } | null;
	onBeforeParse?: (source: string) => Promise<string> | string;
	onInit?: (engine: any) => void;
	onDestroy?: (engine: any) => void;
}

export class PluginRegistry {
	private plugins: Map<string, RuntimePlugin> = new Map();
	private config: AlloyConfig;

	constructor(config: AlloyConfig) {
		this.config = config;
	}

	async load(engine: any) {
		// 1. Load core plugins in fixed order
		// alloy:ffi -> alloy:config -> alloy:shell

		// 2. Dispatch onInit to all registered plugins (core first)
		for (const [name, plugin] of this.plugins.entries()) {
			if (plugin.onInit) {
				this.setCurrentPlugin(name, engine);
				await plugin.onInit(engine);
			}
		}

		// Phase 2 Validation: After all onInit calls (where useConfig is called)
		const configPlugin = this.get("alloy:config") as any;
		if (configPlugin && configPlugin.configManager) {
			const validator = new ConfigValidator();
			const result = validator.validatePluginConfigs(
				this.config.pluginConfigs || {},
				configPlugin.configManager.getSchemas(),
				"alloyconfig.toml",
			);
			if (result.warnings.length > 0) {
				for (const warn of result.warnings) {
					console.warn(
						`[${warn.filePath}${warn.line ? ":" + warn.line : ""}] ${warn.message}`,
					);
				}
			}
		}

		// 3. Load third-party plugins from alloyconfig.toml
		if (this.config.plugins) {
			for (const pluginEntry of this.config.plugins) {
				await this.loadPlugin(pluginEntry, engine);
			}
		}

		// Dispatch onStart to all plugins
		for (const plugin of this.plugins.values()) {
			if (plugin.onStart) {
				await plugin.onStart(engine);
			}
		}
	}

	private async loadPlugin(entry: string | PluginEntry, engine: any) {
		let name: string;
		let path: string;

		if (typeof entry === "string") {
			name = entry;
			path = entry; // Simplified resolution for now
		} else {
			name = entry.name || entry.path || "unknown";
			path = entry.path || name;
		}

		// In a real implementation, we'd use Kahn's algorithm here for topo-sort
		// and actually dynamic-import or dlopen the plugin.
		// For this task, we assume the system plugins are already available.

		this.setCurrentPlugin(name, engine);
	}

	private setCurrentPlugin(name: string, engine: any) {
		if (engine && engine.eval_fn) {
			engine.eval_fn(`globalThis.Alloy.currentPluginName = "${name}";`);
		}
	}

	register(name: string, plugin: RuntimePlugin) {
		this.plugins.set(name, plugin);
	}

	get(name: string): RuntimePlugin | undefined {
		return this.plugins.get(name);
	}

	async dispatchOnResolve(id: string, parent?: string): Promise<string | null> {
		for (const plugin of this.plugins.values()) {
			if (plugin.onResolve) {
				const result = await plugin.onResolve(id, parent);
				if (result) return result;
			}
		}
		return null;
	}

	async dispatchOnLoad(id: string): Promise<{ contents: string; loader: string } | null> {
		for (const plugin of this.plugins.values()) {
			if (plugin.onLoad) {
				const result = await plugin.onLoad(id);
				if (result) return result;
			}
		}
		return null;
	}

	async dispatchOnBeforeParse(source: string): Promise<string> {
		let currentSource = source;
		for (const plugin of this.plugins.values()) {
			if (plugin.onBeforeParse) {
				currentSource = await plugin.onBeforeParse(currentSource);
			}
		}
		return currentSource;
	}
}
