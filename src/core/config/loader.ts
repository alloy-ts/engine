import {
	AlloyConfig,
	BUILTIN_DEFAULTS_LOCAL,
	BUILTIN_DEFAULTS_SERVER,
} from "./config";
import { ConfigValidator, ValidationResult } from "./validator";
import { existsSync, readFileSync } from "node:fs";
import { join, dirname, isAbsolute } from "node:path";
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
			skipUserConfig?: boolean;
		} = {},
	): AlloyConfig {
		const cwd = options.cwd || process.cwd();
		const discovery = this.discover(cwd, options.configPath);

		let userConfig: Partial<AlloyConfig> = {};
		if (!options.skipUserConfig && discovery.userPath) {
			try {
				userConfig = this.parse(discovery.userPath).raw as Partial<AlloyConfig>;
			} catch (e) {
				// Silently skip if file not found, but parse() might throw on syntax error
				if (existsSync(discovery.userPath)) throw e;
			}
		}

		let workspaceConfig: Partial<AlloyConfig> = {};
		if (discovery.workspacePath) {
			try {
				workspaceConfig = this.parse(
					discovery.workspacePath,
				).raw as Partial<AlloyConfig>;
			} catch (e) {
				if (existsSync(discovery.workspacePath)) throw e;
			}
		}

		let merged = this.merge(userConfig, workspaceConfig, options.cliFlags);
		merged = this.interpolate(merged);

		const validator = new ConfigValidator();
		const result = validator.validate(
			merged as Record<string, unknown>,
			discovery.workspacePath || "alloyconfig.toml",
		);
		if (!result.valid) {
			for (const err of result.errors) {
				console.error(
					`[${err.filePath}${err.line ? `:${err.line}` : ""}] field "${
						err.fieldPath
					}": ${err.message}`,
				);
			}
			process.exit(1);
		}

		for (const warn of result.warnings) {
			console.warn(
				`[${warn.filePath}${warn.line ? `:${warn.line}` : ""}] ${warn.message}`,
			);
		}

		return this.injectDefaults(merged);
	}

	discover(cwd: string, configPath?: string): DiscoveryResult {
		let workspacePath: string | null = null;
		if (configPath) {
			workspacePath = isAbsolute(configPath) ? configPath : join(cwd, configPath);
		} else {
			let curr = cwd;
			while (true) {
				const p = join(curr, "alloyconfig.toml");
				if (existsSync(p)) {
					workspacePath = p;
					break;
				}
				if (existsSync(join(curr, "package.json"))) {
					break;
				}
				const parent = dirname(curr);
				if (parent === curr) break;
				curr = parent;
			}
		}

		let userPath: string | null = null;
		const xdg = process.env.XDG_CONFIG_HOME;
		const p1 = xdg ? join(xdg, ".alloyconfig.toml") : null;
		const p2 = join(homedir(), ".alloyconfig.toml");

		if (p1 && existsSync(p1)) {
			userPath = p1;
		} else if (existsSync(p2)) {
			userPath = p2;
		}

		return { workspacePath, userPath };
	}

	parse(path: string): ParseResult {
		try {
			const content = readFileSync(path, "utf-8");
			const raw = TOML.parse(content) as Record<string, unknown>;
			return { raw, filePath: path };
		} catch (e: any) {
			if (e.line && e.column) {
				console.error(`error: ${path}:${e.line}:${e.column}: ${e.message}`);
			} else {
				console.error(`error: ${path}: ${e.message}`);
			}
			process.exit(1);
		}
	}

	merge(
		user: Partial<AlloyConfig>,
		workspace: Partial<AlloyConfig>,
		cliFlags?: Partial<AlloyConfig>,
	): Partial<AlloyConfig> {
		const target =
			cliFlags?.target ||
			workspace.target ||
			user.target ||
			BUILTIN_DEFAULTS_LOCAL.target;
		const defaults =
			target === "local-first" ? BUILTIN_DEFAULTS_LOCAL : BUILTIN_DEFAULTS_SERVER;

		const merged: any = { ...defaults };

		const applyLayer = (layer: any) => {
			for (const key of Object.keys(layer)) {
				if (key === "define") {
					merged[key] = { ...merged[key], ...layer[key] };
				} else if (key === "install") {
					const layerInstall = layer[key];
					if (!merged.install) merged.install = {};
					for (const iKey of Object.keys(layerInstall)) {
						if (iKey === "scopes") {
							merged.install.scopes = {
								...merged.install.scopes,
								...layerInstall.scopes,
							};
						} else {
							merged.install[iKey] = layerInstall[iKey];
						}
					}
				} else {
					merged[key] = layer[key];
				}
			}
		};

		applyLayer(user);
		applyLayer(workspace);
		if (cliFlags) applyLayer(cliFlags);

		return merged;
	}

	interpolate(config: Partial<AlloyConfig>): Partial<AlloyConfig> {
		const walk = (obj: any): any => {
			if (typeof obj === "string") {
				return obj.replace(/\$([A-Z0-9_]+)|\$\{([A-Z0-9_]+)\}/gi, (match, p1, p2) => {
					const varName = p1 || p2;
					return process.env[varName] || "";
				});
			}
			if (Array.isArray(obj)) {
				return obj.map(walk);
			}
			if (obj !== null && typeof obj === "object") {
				const newObj: any = {};
				for (const key of Object.keys(obj)) {
					newObj[key] = walk(obj[key]);
				}
				return newObj;
			}
			return obj;
		};

		return walk(config);
	}

	injectDefaults(config: Partial<AlloyConfig>): AlloyConfig {
		const target = config.target || "local-first";
		const defaults =
			target === "local-first" ? BUILTIN_DEFAULTS_LOCAL : BUILTIN_DEFAULTS_SERVER;

		// The merge already started with defaults, but ensure all fields are present
		// Actually, target-dependent defaults should be handled carefully.
		const result = { ...config } as any;

		if (target === "local-first") {
			if (result.disableWasmEdge === undefined) result.disableWasmEdge = false;
			if (result.useContainers === undefined) result.useContainers = true;
		} else {
			if (result.disableWasmEdge === undefined) result.disableWasmEdge = true;
			if (result.useContainers === undefined) result.useContainers = false;
		}

		return result as AlloyConfig;
	}
}
