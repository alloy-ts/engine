import { existsSync, readFileSync } from "node:fs";
import { join, dirname, isAbsolute } from "node:path";
import { homedir } from "node:os";
import * as TOML from "@iarna/toml";
import { AlloyConfig, BUILTIN_DEFAULTS } from "./config";
import { ConfigValidator } from "./validator";

export interface DiscoveryResult {
	workspacePath: string | null;
	userPath: string | null;
}

export interface ParseResult {
	raw: Record<string, unknown>;
	filePath: string;
}

export interface ConfigLoaderOptions {
	cwd?: string; // override project root (default: process.cwd())
	configPath?: string; // --config flag: explicit WorkspaceConfig path
	cliFlags?: Partial<AlloyConfig>; // CLI flag overrides
	excludeUserConfig?: boolean; // for `alloy run`
}

export class ConfigLoader {
	load(options: ConfigLoaderOptions = {}): AlloyConfig {
		const cwd = options.cwd || process.cwd();

		// Stage 1: Discovery
		const { workspacePath, userPath } = this.discover(cwd, options.configPath);

		// Stage 2: TOML Parsing
		let userRaw: Record<string, unknown> = {};
		if (userPath && !options.excludeUserConfig) {
			userRaw = this.parse(userPath).raw;
		}

		let workspaceRaw: Record<string, unknown> = {};
		if (workspacePath) {
			workspaceRaw = this.parse(workspacePath).raw;
		}

		// Stage 3: Three-Level Merge
		const merged = this.merge(userRaw, workspaceRaw, options.cliFlags);

		// Stage 4: Env Var Interpolation
		const interpolated = this.interpolate(merged);

		// Stage 5: Schema Validation
		const validator = new ConfigValidator();
		const result = validator.validate(
			interpolated,
			workspacePath || "alloyconfig.toml",
			workspacePath ? readFileSync(workspacePath, "utf-8") : undefined,
		);

		for (const warning of result.warnings) {
			console.warn(`[${warning.filePath}:${warning.line || ""}] ${warning.message}`);
		}

		if (!result.valid) {
			for (const error of result.errors) {
				console.error(`[${error.filePath}:${error.line || ""}] ${error.message}`);
			}
			process.exit(1);
		}

		// Stage 6: Default Injection
		return this.injectDefaults(interpolated);
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
				const next = dirname(curr);
				if (next === curr) break;
				curr = next;
			}
		}

		let userPath: string | null = null;
		const xdg = process.env.XDG_CONFIG_HOME;
		const paths = [];
		if (xdg) paths.push(join(xdg, ".alloyconfig.toml"));
		paths.push(join(homedir(), ".alloyconfig.toml"));

		for (const p of paths) {
			if (existsSync(p)) {
				userPath = p;
				break;
			}
		}

		return { workspacePath, userPath };
	}

	parse(path: string): ParseResult {
		const content = readFileSync(path, "utf-8");
		return this.parseString(content, path);
	}

	parseString(content: string, path = "alloyconfig.toml"): ParseResult {
		try {
			const raw = TOML.parse(content) as Record<string, unknown>;
			return { raw, filePath: path };
		} catch (err: any) {
			if (err.line && err.column) {
				console.error(`error: ${path}:${err.line}:${err.column}: ${err.message}`);
			} else {
				console.error(`error: ${path}: ${err.message}`);
			}
			process.exit(1);
		}
	}

	merge(
		user: Record<string, any>,
		workspace: Record<string, any>,
		cliFlags?: Partial<AlloyConfig>,
	): Record<string, any> {
		const target = cliFlags?.target || workspace.target || user.target || "local-first";
		const defaults =
			target === "local-first"
				? BUILTIN_DEFAULTS["local-first"]
				: BUILTIN_DEFAULTS["server-first"];

		const merged: any = JSON.parse(JSON.stringify(defaults));

		const applyLayer = (layer: Record<string, any>) => {
			for (const key of Object.keys(layer)) {
				if (key === "define") {
					merged[key] = { ...(merged[key] || {}), ...layer[key] };
				} else if (key === "install" && layer.install?.scopes) {
					const newInstall = { ...layer.install };
					newInstall.scopes = {
						...(merged.install?.scopes || {}),
						...layer.install.scopes,
					};
					merged.install = newInstall;
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

	interpolate(raw: Record<string, any>): Record<string, any> {
		const traverse = (obj: any): any => {
			if (typeof obj === "string") {
				return obj.replace(
					/\$([A-Z0-9_]+)|\$\{([A-Z0-9_]+)\}/gi,
					(_, p1, p2) => {
						const varName = p1 || p2;
						return process.env[varName] || "";
					},
				);
			}
			if (Array.isArray(obj)) {
				return obj.map(traverse);
			}
			if (obj !== null && typeof obj === "object") {
				const result: Record<string, any> = {};
				for (const key of Object.keys(obj)) {
					result[key] = traverse(obj[key]);
				}
				return result;
			}
			return obj;
		};

		return traverse(raw);
	}

	injectDefaults(raw: Record<string, any>): AlloyConfig {
		const target = raw.target || "local-first";
		const defaults =
			target === "local-first"
				? BUILTIN_DEFAULTS["local-first"]
				: BUILTIN_DEFAULTS["server-first"];

		// Target-dependent defaults that aren't just field replacements
		if (raw.disableWasmEdge === undefined) {
			raw.disableWasmEdge = target === "server-first";
		}
		if (raw.useContainers === undefined) {
			raw.useContainers = target === "local-first";
		}

		return { ...defaults, ...raw } as AlloyConfig;
	}
}
