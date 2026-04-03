import * as fs from "node:fs";
import * as path from "node:path";
import * as TOML from "smol-toml";
import { type AlloyConfig, BUILTIN_DEFAULTS } from "./config";
import { ConfigValidator } from "./validator";

export interface DiscoveryResult {
	workspacePath: string | null;
	userPath: string | null;
}

export interface ConfigLoaderOptions {
	cwd?: string;
	configPath?: string;
	cliFlags?: Partial<AlloyConfig>;
}

export class ConfigLoader {
	load(options: ConfigLoaderOptions = {}): AlloyConfig {
		const cwd = options.cwd || process.cwd();
		const discovery = this.discover(cwd, options.configPath);

		let userConfig: Partial<AlloyConfig> = {};
		if (discovery.userPath) {
			userConfig = this.parse(discovery.userPath);
		}

		let workspaceConfig: Partial<AlloyConfig> = {};
		if (discovery.workspacePath) {
			workspaceConfig = this.parse(discovery.workspacePath);
		}

		const merged = this.merge(userConfig, workspaceConfig, options.cliFlags);
		const interpolated = this.interpolate(merged);

		const validator = new ConfigValidator();
		const validation = validator.validate(
			interpolated as Record<string, unknown>,
			discovery.workspacePath || "builtin",
		);

		if (!validation.valid) {
			for (const err of validation.errors) {
				console.error(`[${err.filePath}] field "${err.fieldPath}": ${err.message}`);
			}
			process.exit(1);
		}

		for (const warn of validation.warnings) {
			console.warn(`[${warn.filePath}] field "${warn.fieldPath}": ${warn.message}`);
		}

		return this.injectDefaults(interpolated);
	}

	discover(cwd: string, configPath?: string): DiscoveryResult {
		let workspacePath: string | null = null;
		if (configPath) {
			workspacePath = path.resolve(configPath);
		} else {
			const local = path.join(cwd, "alloyconfig.toml");
			if (fs.existsSync(local)) {
				workspacePath = local;
			} else {
				// Walk up
				let curr = cwd;
				while (curr !== path.parse(curr).root) {
					if (fs.existsSync(path.join(curr, "package.json"))) {
						const upLocal = path.join(curr, "alloyconfig.toml");
						if (fs.existsSync(upLocal)) {
							workspacePath = upLocal;
						}
						break;
					}
					curr = path.dirname(curr);
				}
			}
		}

		const home = process.env.HOME || "";
		const xdg = process.env.XDG_CONFIG_HOME;
		let userPath: string | null = null;

		if (xdg) {
			const p = path.join(xdg, ".alloyconfig.toml");
			if (fs.existsSync(p)) userPath = p;
		}

		if (!userPath && home) {
			const p = path.join(home, ".alloyconfig.toml");
			if (fs.existsSync(p)) userPath = p;
		}

		return { workspacePath, userPath };
	}

	parse(filePath: string): Partial<AlloyConfig> {
		try {
			const content = fs.readFileSync(filePath, "utf-8");
			return TOML.parse(content) as Partial<AlloyConfig>;
		} catch (e: any) {
			console.error(`error: ${filePath}: ${e.message}`);
			process.exit(1);
		}
	}

	merge(
		user: Partial<AlloyConfig>,
		workspace: Partial<AlloyConfig>,
		cliFlags?: Partial<AlloyConfig>,
	): Partial<AlloyConfig> {
		const merged: any = { ...BUILTIN_DEFAULTS };

		const apply = (config: Partial<AlloyConfig>) => {
			for (const [key, value] of Object.entries(config)) {
				if (key === "define") {
					merged.define = { ...merged.define, ...value };
				} else if (key === "install" && (value as any).scopes) {
					merged.install = {
						...merged.install,
						...value,
						scopes: { ...merged.install?.scopes, ...(value as any).scopes },
					};
				} else {
					merged[key] = value;
				}
			}
		};

		apply(user);
		apply(workspace);
		if (cliFlags) apply(cliFlags);

		return merged;
	}

	interpolate(raw: Partial<AlloyConfig>): Partial<AlloyConfig> {
		const recurse = (val: any): any => {
			if (typeof val === "string") {
				return val.replace(/\$(\w+)|\${(\w+)}/g, (_, v1, v2) => {
					const name = v1 || v2;
					return process.env[name] || "";
				});
			}
			if (Array.isArray(val)) {
				return val.map(recurse);
			}
			if (val !== null && typeof val === "object") {
				const res: any = {};
				for (const [k, v] of Object.entries(val)) {
					res[k] = recurse(v);
				}
				return res;
			}
			return val;
		};

		return recurse(raw);
	}

	injectDefaults(raw: Partial<AlloyConfig>): AlloyConfig {
		const config = { ...raw } as AlloyConfig;
		const target = config.target || "local-first";

		if (config.disableWasmEdge === undefined) {
			config.disableWasmEdge = target === "server-first";
		}
		if (config.useContainers === undefined) {
			config.useContainers = target === "local-first";
		}

		return config;
	}
}
