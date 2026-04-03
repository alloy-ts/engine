import { existsSync, readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { homedir } from "node:os";
import TOML from "@iarna/toml";
import { DEFAULT_CONFIG, type AlloyConfig } from "./schema";

export function interpolateEnv(value: string): string {
	return value.replace(/\$(\w+)|\${(\w+)}/g, (_, v1, v2) => {
		const varName = v1 || v2;
		return process.env[varName] || "";
	});
}

function interpolateRecursive(obj: any): any {
	if (typeof obj === "string") {
		return interpolateEnv(obj);
	}
	if (Array.isArray(obj)) {
		return obj.map((item) => interpolateRecursive(item));
	}
	if (typeof obj === "object" && obj !== null) {
		const newObj: any = {};
		for (const key in obj) {
			newObj[key] = interpolateRecursive(obj[key]);
		}
		return newObj;
	}
	return obj;
}

export function findProjectRoot(cwd: string): string {
	let current = cwd;
	while (true) {
		if (existsSync(join(current, "package.json")) || existsSync(join(current, "alloyconfig.toml"))) {
			return current;
		}
		const parent = dirname(current);
		if (parent === current) {
			return cwd;
		}
		current = parent;
	}
}

export class ConfigLoader {
	load(explicitConfigPath?: string): AlloyConfig {
		const globalConfig = this.loadGlobal();
		const localConfig = this.loadLocal(explicitConfigPath);

		const merged = this.shallowMerge(globalConfig, localConfig);
		const interpolated = interpolateRecursive(merged);

		const config: AlloyConfig = { ...DEFAULT_CONFIG, ...interpolated };

		// Apply conditional defaults
		if (config.target === "server-first") {
			if (config.disableWasmEdge === undefined) config.disableWasmEdge = true;
			if (config.useContainers === undefined) config.useContainers = false;
		} else {
			// local-first
			if (config.disableWasmEdge === undefined) config.disableWasmEdge = false;
			if (config.useContainers === undefined) config.useContainers = true;
		}

		return config;
	}

	private loadGlobal(): Partial<AlloyConfig> {
		const xdgConfig = process.env.XDG_CONFIG_HOME;
		const globalPath = xdgConfig
			? join(xdgConfig, ".alloyconfig.toml")
			: join(homedir(), ".alloyconfig.toml");

		return this.parseFile(globalPath);
	}

	private loadLocal(explicitPath?: string): Partial<AlloyConfig> {
		if (explicitPath) {
			return this.parseFile(explicitPath);
		}
		const root = findProjectRoot(process.cwd());
		const localPath = join(root, "alloyconfig.toml");
		return this.parseFile(localPath);
	}

	private parseFile(path: string): Partial<AlloyConfig> {
		if (!existsSync(path)) {
			return {};
		}
		try {
			const content = readFileSync(path, "utf-8");
			return TOML.parse(content) as any;
		} catch (e: any) {
			console.error(`Error parsing TOML at ${path}: ${e.message}`);
			process.exit(1);
		}
	}

	private shallowMerge(
		global: Partial<AlloyConfig>,
		local: Partial<AlloyConfig>,
	): Partial<AlloyConfig> {
		const merged = { ...global };
		for (const key in local) {
			// @ts-ignore
			merged[key] = local[key];
		}
		return merged;
	}
}
