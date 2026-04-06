import { AlloyConfig } from "./config/config";
import { ConfigLoader } from "./config/loader";
import { PrettyPrinter } from "./config/pretty_printer";
import { writeFileSync, existsSync } from "node:fs";
import { join } from "node:path";

export class ConfigManager {
	private config: AlloyConfig;
	private loader: ConfigLoader;
	private printer: PrettyPrinter;
	private workspacePath: string | null = null;
	private schemas: Record<string, any> = {};

	constructor(initialConfig?: AlloyConfig) {
		this.loader = new ConfigLoader();
		this.printer = new PrettyPrinter();
		if (initialConfig) {
			this.config = initialConfig;
		} else {
			this.config = this.loader.load();
		}

		// Try to discover the workspace path for persistence
		const discovery = this.loader.discover(process.cwd());
		this.workspacePath = discovery.workspacePath;
	}

	get(keyPath: string): any {
		const parts = keyPath.split(".");
		let current: any = this.config;
		for (const part of parts) {
			if (current === undefined || current === null) return undefined;
			current = current[part];
		}
		return current;
	}

	set(keyPath: string, value: any, options: { persist?: boolean } = {}): void {
		const parts = keyPath.split(".");
		let current: any = this.config;
		for (let i = 0; i < parts.length - 1; i++) {
			const part = parts[i];
			if (current[part] === undefined) {
				current[part] = {};
			}
			current = current[part];
		}
		current[parts[parts.length - 1]] = value;

		if (options.persist && this.workspacePath) {
			this.persist();
		}
	}

	registerSchema(pluginName: string, schema: any): void {
		if (this.schemas[pluginName]) {
			// Merge schema (Requirement 20.5)
			const existing = this.schemas[pluginName].configs;
			for (const key of Object.keys(schema.configs)) {
				if (existing[key]) {
					console.warn(
						`[alloy:config] duplicate config entry "${key}" for plugin "${pluginName}"; later declaration wins`,
					);
				}
				existing[key] = schema.configs[key];
			}
		} else {
			this.schemas[pluginName] = schema;
		}

		// Initialize defaults if not present or updating with a new schema (Requirement 20.5)
		if (schema.configs) {
			for (const [key, def] of Object.entries(schema.configs) as [
				string,
				any,
			][]) {
				const fullPath = `pluginConfigs.${pluginName}.${key}`;
				// For new entries or merging schemas, we apply the default if current is undefined
				// or if it's explicitly a re-registration with a new default
				if (this.get(fullPath) === undefined && def.default !== undefined) {
					this.set(fullPath, def.default);
				}
			}
		}
	}

	private persist(): void {
		if (!this.workspacePath) return;
		const toml = this.printer.print(this.config);
		writeFileSync(this.workspacePath, toml, "utf-8");
	}

	getConfig(): AlloyConfig {
		return this.config;
	}

	getSchemas(): Map<string, any> {
		const map = new Map<string, any>();
		for (const [name, schema] of Object.entries(this.schemas)) {
			map.set(name, schema);
		}
		return map;
	}
}
