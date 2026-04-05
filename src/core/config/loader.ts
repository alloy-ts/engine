import { AlloyConfig, DEFAULTS_LOCAL, DEFAULTS_SERVER } from "./config";
import { ConfigValidator } from "./validator";
import { existsSync, readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { homedir } from "node:os";

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
		} = {},
	): AlloyConfig {
		const cwd = options.cwd || process.cwd();
		const discovery = this.discover(cwd, options.configPath);

		let userConfig: Partial<AlloyConfig> = {};
		if (discovery.userPath && existsSync(discovery.userPath)) {
			userConfig = this.parse(discovery.userPath);
		}

		let workspaceConfig: Partial<AlloyConfig> = {};
		if (discovery.workspacePath && existsSync(discovery.workspacePath)) {
			workspaceConfig = this.parse(discovery.workspacePath);
		}

		let config = this.merge(userConfig, workspaceConfig, options.cliFlags);
		config = this.interpolate(config);

		const validator = new ConfigValidator();
		const result = validator.validate(config);
		if (!result.valid) {
			console.warn("Configuration validation errors:", result.errors);
		}

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
					// Also stop if package.json found but no alloyconfig.toml yet
					const p2 = join(curr, "alloyconfig.toml");
					if (existsSync(p2)) workspacePath = p2;
					break;
				}
				curr = dirname(curr);
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

	merge(
		user: Partial<AlloyConfig>,
		workspace: Partial<AlloyConfig>,
		cliFlags?: Partial<AlloyConfig>,
	): Partial<AlloyConfig> {
		const target = workspace.target || user.target || "local-first";
		const defaults =
			target === "local-first" ? DEFAULTS_LOCAL : DEFAULTS_SERVER;

		let merged: any = { ...defaults };

		const apply = (layer: any) => {
			for (const key of Object.keys(layer)) {
				if (key === "define" || key === "install") {
					// simplified logic for special tables
					merged[key] = { ...merged[key], ...layer[key] };
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

	parse(path: string): any {
		const content = readFileSync(path, "utf-8");
		const result: any = {};
		const lines = content.split("\n");
		let currentSection: string | null = null;

		for (const line of lines) {
			const trimmed = line.trim();
			if (!trimmed || trimmed.startsWith("#")) continue;

			if (trimmed.startsWith("[") && trimmed.endsWith("]")) {
				currentSection = trimmed.substring(1, trimmed.length - 1);
				result[currentSection] = result[currentSection] || {};
				continue;
			}

			const [key, ...rest] = trimmed.split("=");
			if (key && rest.length > 0) {
				let value: any = rest.join("=").trim();
				if (value.startsWith('"') && value.endsWith('"')) {
					value = value.substring(1, value.length - 1);
				} else if (value === "true") {
					value = true;
				} else if (value === "false") {
					value = false;
				} else if (!isNaN(Number(value))) {
					value = Number(value);
				} else if (value.startsWith("[") && value.endsWith("]")) {
					value = value
						.substring(1, value.length - 1)
						.split(",")
						.map((s: string) => s.trim().replace(/^"|"$/g, ""));
				}

				if (currentSection) {
					result[currentSection][key.trim()] = value;
				} else {
					result[key.trim()] = value;
				}
			}
		}
		return result;
	}

	interpolate(config: any): any {
		const json = JSON.stringify(config);
		const interpolated = json.replace(
			/\$([A-Z0-9_]+)|\$\{([A-Z0-9_]+)\}/gi,
			(match, p1, p2) => {
				const varName = p1 || p2;
				return process.env[varName] || "";
			},
		);
		return JSON.parse(interpolated);
	}

	injectDefaults(config: Partial<AlloyConfig>): AlloyConfig {
		const target = config.target || "local-first";
		const defaults =
			target === "local-first" ? DEFAULTS_LOCAL : DEFAULTS_SERVER;
		return { ...defaults, ...config } as AlloyConfig;
	}
}
