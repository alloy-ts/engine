import {
	AlloyConfig,
	BUILTIN_DEFAULTS_LOCAL,
	BUILTIN_DEFAULTS_SERVER,
} from "./config";
import { ConfigValidator } from "./validator";
import { existsSync, readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { homedir } from "node:os";
import * as TOML from "@iarna/toml";

export interface DiscoveryResult {
	workspacePath: string | null;
	userPath: string | null;
}

export interface ParseResult {
	raw: Record<string, unknown>;
	filePath: string;
}

export class ConfigLoader {
	load(
		options: {
			cwd?: string;
			configPath?: string;
			cliFlags?: Partial<AlloyConfig>;
			excludeUserConfig?: boolean;
		} = {},
	): AlloyConfig {
		const cwd = options.cwd || process.cwd();

		// Stage 1: Discovery
		const discovery = this.discover(cwd, options.configPath);

		// Stage 2: Parsing
		let userConfig: Record<string, unknown> = {};
		if (
			!options.excludeUserConfig &&
			discovery.userPath &&
			existsSync(discovery.userPath)
		) {
			userConfig = this.parse(discovery.userPath).raw;
		}

		let workspaceConfig: Record<string, unknown> = {};
		if (discovery.workspacePath && existsSync(discovery.workspacePath)) {
			workspaceConfig = this.parse(discovery.workspacePath).raw;
		}

		// Stage 3: Merge
		let config = this.merge(userConfig, workspaceConfig, options.cliFlags);

		// Stage 4: Interpolation
		config = this.interpolate(config);

		// Stage 5: Validation
		const validator = new ConfigValidator();
		const validationResult = validator.validate(
			config,
			discovery.workspacePath || "alloyconfig.toml",
		);
		if (!validationResult.valid) {
			// In a real CLI, we would print all errors and process.exit(1)
			console.error("Configuration validation failed:");
			for (const err of validationResult.errors) {
				console.error(
					`[${err.filePath}${err.line ? ":" + err.line : ""}] field "${err.fieldPath}": ${err.message}`,
				);
			}
			throw new Error("Invalid configuration");
		}
		if (validationResult.warnings.length > 0) {
			for (const warn of validationResult.warnings) {
				console.warn(
					`[${warn.filePath}${warn.line ? ":" + warn.line : ""}] ${warn.message}`,
				);
			}
		}

		// Stage 6: Inject Defaults
		return this.injectDefaults(config);
	}

	discover(cwd: string, configPath?: string): DiscoveryResult {
		let workspacePath: string | null = null;
		if (configPath) {
			workspacePath = join(cwd, configPath);
		} else {
			let curr = cwd;
			while (curr !== "/") {
				const p = join(curr, "alloyconfig.toml");
				if (existsSync(p)) {
					workspacePath = p;
					break;
				}
				if (existsSync(join(curr, "package.json"))) {
					const p2 = join(curr, "alloyconfig.toml");
					if (existsSync(p2)) workspacePath = p2;
					break;
				}
				const parent = dirname(curr);
				if (parent === curr) break;
				curr = parent;
			}
		}

		let userPath: string | null = null;
		const xdg = process.env.XDG_CONFIG_HOME;
		const p1 = xdg
			? join(xdg, ".alloyconfig.toml")
			: join(homedir(), ".alloyconfig.toml");
		if (existsSync(p1)) {
			userPath = p1;
		} else {
			const p2 = join(homedir(), ".alloyconfig.toml");
			if (existsSync(p2)) userPath = p2;
		}

		return { workspacePath, userPath };
	}

	parse(path: string): ParseResult {
		const content = readFileSync(path, "utf-8");
		try {
			const raw = TOML.parse(content) as Record<string, any>;
			// Handle TOML naming conflict: [plugins.<name>] vs [[plugins]]
			if (raw.plugins && !Array.isArray(raw.plugins)) {
				raw.pluginConfigs = raw.plugins;
				delete raw.plugins;
			}
			return { raw, filePath: path };
		} catch (e: any) {
			const line = e.line || (e.row + 1) || 0;
			const col = e.column || (e.col + 1) || 0;
			console.error(`error: ${path}:${line}:${col}: ${e.message}`);
			throw e;
		}
	}

	merge(
		user: Partial<AlloyConfig>,
		workspace: Partial<AlloyConfig>,
		cliFlags?: Partial<AlloyConfig>,
	): Partial<AlloyConfig> {
		const target = workspace.target || user.target || "local-first";
		const defaults =
			target === "local-first" ? BUILTIN_DEFAULTS_LOCAL : BUILTIN_DEFAULTS_SERVER;

		let merged: any = JSON.parse(JSON.stringify(defaults));

		const apply = (layer: any) => {
			for (const key of Object.keys(layer)) {
				if (key === "define" || key === "install") {
					if (key === "install" && layer.install?.scopes) {
						merged.install = merged.install || {};
						merged.install.scopes = {
							...merged.install.scopes,
							...layer.install.scopes,
						};
						// Copy other install fields
						for (const k of Object.keys(layer.install)) {
							if (k !== "scopes") merged.install[k] = layer.install[k];
						}
					} else if (key === "define") {
						merged.define = { ...merged.define, ...layer.define };
					} else {
						merged[key] = layer[key];
					}
				} else {
					merged[key] = layer[key];
				}
			}
		};

		apply(user);
		apply(workspace);
		if (cliFlags) apply(cliFlags);

		return merged;
	}

	interpolate(config: any): any {
		const interpolateValue = (val: any): any => {
			if (typeof val === "string") {
				return val.replace(/\$([A-Z0-9_]+)|\$\{([A-Z0-9_]+)\}/gi, (_, p1, p2) => {
					const varName = p1 || p2;
					return process.env[varName] || "";
				});
			}
			if (Array.isArray(val)) {
				return val.map(interpolateValue);
			}
			if (typeof val === "object" && val !== null) {
				const res: any = {};
				for (const key of Object.keys(val)) {
					res[key] = interpolateValue(val[key]);
				}
				return res;
			}
			return val;
		};

		return interpolateValue(config);
	}

	injectDefaults(config: Partial<AlloyConfig>): AlloyConfig {
		const target = config.target || "local-first";
		const defaults =
			target === "local-first" ? BUILTIN_DEFAULTS_LOCAL : BUILTIN_DEFAULTS_SERVER;

		// Re-apply target-dependent defaults for useContainers/disableWasmEdge if not set
		const final = { ...config };
		if (final.disableWasmEdge === undefined)
			final.disableWasmEdge = defaults.disableWasmEdge;
		if (final.useContainers === undefined)
			final.useContainers = defaults.useContainers;

		// Merge inline config from plugins array (Requirement 21.7)
		if (final.plugins && Array.isArray(final.plugins)) {
			final.pluginConfigs = final.pluginConfigs || {};
			for (const plugin of final.plugins) {
				if (typeof plugin === "object" && plugin.name && plugin.config) {
					final.pluginConfigs[plugin.name] = {
						...(final.pluginConfigs[plugin.name] || {}),
						...(plugin.config as any),
					};
				}
			}
		}

		return final as AlloyConfig;
	}
}
