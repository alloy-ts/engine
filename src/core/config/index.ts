import { ConfigLoader } from "./loader";
import { ConfigValidator } from "./validator";
import type { AlloyConfig } from "./schema";

export * from "./schema";
export * from "./loader";
export * from "./validator";
export * from "./pretty_printer";

let cachedConfig: AlloyConfig | null = null;

export function getConfig(explicitPath?: string): AlloyConfig {
	if (cachedConfig && !explicitPath) return cachedConfig;

	const loader = new ConfigLoader();
	const validator = new ConfigValidator();
	const config = loader.load(explicitPath);

	const errors = validator.validate(config);
	if (errors.length > 0) {
		for (const error of errors) {
			console.error(`[Config Error] ${error}`);
		}
		process.exit(1);
	}

	if (!explicitPath) cachedConfig = config;
	return config;
}
