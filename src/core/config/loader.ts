// src/core/config/loader.ts
import {
	type AlloyConfig,
	BUILTIN_DEFAULTS,
	DEFAULTS_LOCAL,
	DEFAULTS_SERVER,
} from "./config";
import { ConfigValidator } from "./validator";
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

export interface ConfigLoaderOptions {
	cwd?: string;
	configPath?: string;
	cliFlags?: Partial<AlloyConfig>;
	excludeUserConfig?: boolean; // for alloy run
}

export class ConfigLoader {
	load(options: ConfigLoaderOptions = {}): AlloyConfig {
		const cwd = options.cwd || process.cwd();
		const discovery = this.discover(cwd, options.configPath);

		let userRaw: Record<string, unknown> = {};
		if (!options.excludeUserConfig && discovery.userPath) {
			const parseResult = this.parse(discovery.userPath);
			userRaw = parseResult.raw;
		}

		let workspaceRaw: Record<string, unknown> = {};
		if (discovery.workspacePath) {
			const parseResult = this.parse(discovery.workspacePath);
			workspaceRaw = parseResult.raw;
		}

		const mergedRaw = this.merge(
			userRaw as Partial<AlloyConfig>,
			workspaceRaw as Partial<AlloyConfig>,
			options.cliFlags,
		);

		const interpolated = this.interpolate(mergedRaw);

		const validator = new ConfigValidator();
		const result = validator.validate(
			interpolated as Record<string, unknown>,
			discovery.workspacePath || "alloyconfig.toml",
		);

		if (!result.valid) {
			for (const err of result.errors) {
				console.error(
					`error: ${err.filePath}:${err.line || ""}: ${err.message}`,
				);
			}
			process.exit(1);
		}

		for (const warn of result.warnings) {
			console.warn(
				`warning: ${warn.filePath}:${warn.line || ""}: ${warn.message}`,
			);
		}

		return this.injectDefaults(interpolated as Partial<AlloyConfig>);
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
		const p1 = xdg
			? join(xdg, ".alloyconfig.toml")
			: join(homedir(), ".alloyconfig.toml");

		if (xdg && existsSync(p1)) {
			userPath = p1;
		} else {
			const p2 = join(homedir(), ".alloyconfig.toml");
			if (existsSync(p2)) {
				userPath = p2;
			}
		}

		return { workspacePath, userPath };
	}

	parse(path: string): ParseResult {
		try {
			const content = readFileSync(path, "utf-8");
			const raw = TOML.parse(content) as Record<string, unknown>;
			return { raw, filePath: path };
		} catch (e: any) {
			const line = e.line || "";
			const col = e.column || "";
			console.error(`error: ${path}:${line}:${col}: ${e.message}`);
			process.exit(1);
		}
	}

	merge(
		user: Partial<AlloyConfig>,
		workspace: Partial<AlloyConfig>,
		cliFlags?: Partial<AlloyConfig>,
	): Partial<AlloyConfig> {
		const merged: any = { ...BUILTIN_DEFAULTS };

		const applyLayer = (layer: any) => {
			for (const key in layer) {
				if (key === "define" || (key === "install" && layer.install?.scopes)) {
					if (key === "define") {
						merged.define = { ...merged.define, ...layer.define };
					} else {
						const scopes = {
							...merged.install?.scopes,
							...layer.install.scopes,
						};
						merged.install = { ...merged.install, ...layer.install, scopes };
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

	interpolate(raw: Partial<AlloyConfig>): Partial<AlloyConfig> {
		const interpolateString = (val: string): string => {
			return val.replace(/\$([A-Z0-9_]+)|\$\{([A-Z0-9_]+)\}/gi, (_, p1, p2) => {
				const varName = p1 || p2;
				return process.env[varName] || "";
			});
		};

		const walk = (obj: any): any => {
			if (typeof obj === "string") {
				return interpolateString(obj);
			}
			if (Array.isArray(obj)) {
				return obj.map(walk);
			}
			if (obj !== null && typeof obj === "object") {
				const result: any = {};
				for (const key in obj) {
					result[key] = walk(obj[key]);
				}
				return result;
			}
			return obj;
		};

		return walk(raw);
	}

	injectDefaults(raw: Partial<AlloyConfig>): AlloyConfig {
		const target = raw.target || "local-first";
		const defaults = target === "local-first" ? DEFAULTS_LOCAL : DEFAULTS_SERVER;

		// We need to be careful with target-dependent defaults that might have been explicitly set
		const config = { ...defaults, ...raw };

		if (raw.disableWasmEdge === undefined) {
			config.disableWasmEdge = target === "server-first";
		}
		if (raw.useContainers === undefined) {
			config.useContainers = target === "local-first";
		}

		return config as AlloyConfig;
	}
}
