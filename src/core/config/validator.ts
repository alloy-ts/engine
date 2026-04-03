import type { AlloyConfig } from "./config";
import { AlloyConfigSchema } from "./schema";

export interface ValidationError {
	kind: "type_error" | "cross_field_error";
	filePath: string;
	line: number | null;
	fieldPath: string;
	message: string;
}

export interface ValidationWarning {
	kind: "unknown_key";
	filePath: string;
	line: number | null;
	fieldPath: string;
	message: string;
}

export interface ValidationResult {
	errors: ValidationError[];
	warnings: ValidationWarning[];
	valid: boolean;
}

export class ConfigValidator {
	validate(raw: Record<string, unknown>, filePath: string): ValidationResult {
		const errors: ValidationError[] = [];
		const warnings: ValidationWarning[] = [];

		const result = AlloyConfigSchema.safeParse(raw);

		if (!result.success) {
			for (const issue of result.error.issues) {
				errors.push({
					kind: "type_error",
					filePath,
					line: null, // Zod doesn't give us line numbers easily from a raw object
					fieldPath: issue.path.join("."),
					message: issue.message,
				});
			}
		}

		// Cross-field rules
		const config = (result.success ? result.data : raw) as Partial<AlloyConfig>;

		if (config.test?.seed !== undefined && config.test?.randomize !== true) {
			warnings.push({
				kind: "unknown_key", // Using this for general warnings
				filePath,
				line: null,
				fieldPath: "test.seed",
				message: "test.seed has no effect without test.randomize: true",
			});
		}

		if (config.run?.shell === "system" && config.target !== "server-first") {
			errors.push({
				kind: "cross_field_error",
				filePath,
				line: null,
				fieldPath: "run.shell",
				message: 'value "system" requires target = "server-first"',
			});
		}

		if (config.useContainers === false && config.target === "local-first") {
			errors.push({
				kind: "cross_field_error",
				filePath,
				line: null,
				fieldPath: "useContainers",
				message: 'cannot be false when target = "local-first"',
			});
		}

		if (config.shellCapabilityImages) {
			const caps = config.shellCapabilities || [];
			for (const cap of Object.keys(config.shellCapabilityImages)) {
				if (!caps.includes(cap)) {
					errors.push({
						kind: "cross_field_error",
						filePath,
						line: null,
						fieldPath: `shellCapabilityImages.${cap}`,
						message: `key "${cap}" not found in shellCapabilities`,
					});
				}
			}
		}

		return {
			errors,
			warnings,
			valid: errors.length === 0,
		};
	}
}
