import { AlloyConfig, DEFAULTS_LOCAL, DEFAULTS_SERVER, BUILTIN_DEFAULTS } from "./config";
import { ConfigValidator } from "./validator";
import { existsSync, readFileSync } from "node:fs";
import { join, dirname, isAbsolute } from "node:path";
import { homedir } from "node:os";
import * as TOML from "@iarna/toml";

export interface DiscoveryResult {
	workspacePath: string | null;
	userPath: string | null;
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
		const discovery = this.discover(cwd, options.configPath);

		let userConfig: Partial<AlloyConfig> = {};
		if (
			!options.excludeUserConfig &&
			discovery.userPath &&
			existsSync(discovery.userPath)
		) {
			userConfig = this.parse(discovery.userPath);
		}

		let workspaceConfig: Partial<AlloyConfig> = {};
		if (discovery.workspacePath && existsSync(discovery.workspacePath)) {
			workspaceConfig = this.parse(discovery.workspacePath);
		}

		let config = this.merge(userConfig, workspaceConfig, options.cliFlags);
		config = this.interpolate(config);

		const validator = new ConfigValidator();
		const result = validator.validate(
			config as Record<string, unknown>,
			discovery.workspacePath || "in-memory",
		);
		if (!result.valid) {
			const errorMsgs = result.errors
				.map((e) => `[${e.filePath}:${e.line || "?"}] field "${e.fieldPath}": ${e.message}`)
				.join("\n");
			throw new Error(`Configuration validation errors:\n${errorMsgs}`);
		}
		if (result.warnings.length > 0) {
			for (const w of result.warnings) {
				console.warn(
					`[${w.filePath}:${w.line || "?"}] ${w.kind === "unknown_key" ? "unknown key" : "warning"} "${w.fieldPath}" — ignored`,
				);
			}
		}

		return this.injectDefaults(config);
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
		if (existsSync(p1)) {
			userPath = p1;
		} else {
			const p2 = join(homedir(), ".alloyconfig.toml");
			if (existsSync(p2)) userPath = p2;
		}

		return { workspacePath, userPath };
	}

	parse(path: string): any {
		try {
			const content = readFileSync(path, "utf-8");
			return TOML.parse(content);
		} catch (e: any) {
			if (e.line && e.column) {
				throw new Error(
					`error: ${path}:${e.line}:${e.column}: ${e.message}`,
				);
			}
			throw e;
		}
	}

	merge(
		user: Partial<AlloyConfig>,
		workspace: Partial<AlloyConfig>,
		cliFlags?: Partial<AlloyConfig>,
	): Partial<AlloyConfig> {
		const target = workspace.target || user.target || "local-first";
		const base = target === "local-first" ? DEFAULTS_LOCAL : DEFAULTS_SERVER;

		const result: any = JSON.parse(JSON.stringify(base));

		const applyLayer = (layer: any) => {
			for (const key of Object.keys(layer)) {
				const val = layer[key];
				if (val === undefined) continue;

				if (key === "define") {
					result.define = { ...(result.define || {}), ...val };
				} else if (key === "install") {
					const existingInstall = result.install || {};
					const newInstall = val || {};
					const mergedInstall = { ...existingInstall, ...newInstall };
					if (existingInstall.scopes || newInstall.scopes) {
						mergedInstall.scopes = {
							...(existingInstall.scopes || {}),
							...(newInstall.scopes || {}),
						};
					}
					result.install = mergedInstall;
				} else {
					result[key] = val;
				}
			}
		};

		applyLayer(user);
		applyLayer(workspace);
		if (cliFlags) {
			applyLayer(cliFlags);
		}

		return result;
	}

	interpolate(config: any): any {
		const traverse = (obj: any): any => {
			if (typeof obj === "string") {
				return obj.replace(
					/\$([a-zA-Z_][a-zA-Z0-9_]*)|\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}/g,
					(match, p1, p2) => {
						const varName = p1 || p2;
						return process.env[varName] || "";
					},
				);
			}
			if (Array.isArray(obj)) {
				return obj.map(traverse);
			}
			if (obj !== null && typeof obj === "object") {
				const result: any = {};
				for (const key of Object.keys(obj)) {
					result[key] = traverse(obj[key]);
				}
				return result;
			}
			return obj;
		};

		return traverse(config);
	}

	injectDefaults(config: Partial<AlloyConfig>): AlloyConfig {
		const target = config.target || "local-first";
		const defaults =
			target === "local-first" ? DEFAULTS_LOCAL : DEFAULTS_SERVER;

		// The merge stage already applied defaults, but this ensures a clean final object
		// and handles target-dependent defaults like disableWasmEdge and useContainers
		const final = { ...defaults, ...config } as AlloyConfig;

		if (final.target === "local-first") {
			if (final.disableWasmEdge === undefined) final.disableWasmEdge = false;
			if (final.useContainers === undefined) final.useContainers = true;
		} else {
			if (final.disableWasmEdge === undefined) final.disableWasmEdge = true;
			if (final.useContainers === undefined) final.useContainers = false;
		}

		return final;
	}
}
