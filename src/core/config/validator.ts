import type { AlloyConfig } from "./schema";

export class ConfigValidator {
	validate(config: AlloyConfig): string[] {
		const errors: string[] = [];
		const warnings: string[] = [];

		if (config.target && !["local-first", "server-first"].includes(config.target)) {
			errors.push(`Invalid target: ${config.target}. Must be "local-first" or "server-first".`);
		}

		if (config.target === "local-first" && config.useContainers === false) {
			errors.push("useContainers cannot be false when target is local-first.");
		}

		if (config.run?.shell === "system" && config.target === "local-first") {
			errors.push("run.shell cannot be 'system' when target is local-first.");
		}

		if (config.test?.seed !== undefined && config.test?.randomize !== true) {
			warnings.push("test.seed has no effect without test.randomize: true");
		}

		if (config.loader) {
			const validLoaders = ["jsx", "js", "ts", "tsx", "css", "file", "json", "toml", "wasm", "napi", "base64", "dataurl", "text"];
			for (const ext in config.loader) {
				if (!validLoaders.includes(config.loader[ext])) {
					errors.push(`Invalid loader for extension ${ext}: ${config.loader[ext]}. Valid loaders are: ${validLoaders.join(", ")}`);
				}
			}
		}

		if (config.define) {
			for (const key in config.define) {
				if (typeof config.define[key] !== "string") {
					errors.push(`Define field ${key} must be a string.`);
				}
			}
		}

		for (const w of warnings) {
			console.warn(`[Config Warning] ${w}`);
		}

		return errors;
	}
}
