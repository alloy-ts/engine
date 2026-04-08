// src/core/config/validator.ts

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

		// Check for unknown keys
		this.checkUnknownKeys(raw, AlloyConfigSchema.shape, "", filePath, warnings);

		// Zod validation for types
		const result = AlloyConfigSchema.safeParse(raw);
		if (!result.success) {
			let zodErrors = (result as any).error.errors;
			if (!zodErrors && (result as any).error.message) {
				try {
					zodErrors = JSON.parse((result as any).error.message);
				} catch (_e) {}
			}
			if (zodErrors) {
				for (const error of zodErrors) {
					errors.push({
						kind: "type_error",
						filePath,
						line: null, // TOML parser might provide this, but Zod doesn't directly
						fieldPath: error.path.join("."),
						message: error.message,
					});
				}
			}
		}

		// Cross-field validation
		if (result.success) {
			this.crossFieldValidation(
				result.data as AlloyConfig,
				filePath,
				errors,
				warnings,
			);
		}

		return {
			errors,
			warnings,
			valid: errors.length === 0,
		};
	}

	private checkUnknownKeys(
		obj: any,
		shape: any,
		prefix: string,
		filePath: string,
		warnings: ValidationWarning[],
	) {
		if (typeof obj !== "object" || obj === null || Array.isArray(obj)) return;

		for (const key in obj) {
			const fullPath = prefix ? `${prefix}.${key}` : key;
			if (!(key in shape)) {
				warnings.push({
					kind: "unknown_key",
					filePath,
					line: null,
					fieldPath: fullPath,
					message: `unknown key "${fullPath}" — ignored`,
				});
			} else {
				const nestedShape = shape[key];
				// Simple heuristic for nested objects in Zod
				let shapeToUse: any = null;
				if (nestedShape?._def?.shape) {
					shapeToUse = nestedShape._def.shape;
				} else if (nestedShape?._def?.innerType?._def?.shape) {
					shapeToUse = nestedShape._def.innerType._def.shape;
				}

				if (shapeToUse) {
					const actualShape =
						typeof shapeToUse === "function" ? shapeToUse() : shapeToUse;
					this.checkUnknownKeys(
						obj[key],
						actualShape,
						fullPath,
						filePath,
						warnings,
					);
				}
			}
		}
	}

	private crossFieldValidation(
		config: AlloyConfig,
		filePath: string,
		errors: ValidationError[],
		warnings: ValidationWarning[],
	) {
		// 1. test.seed set but test.randomize !== true
		if (config.test?.seed !== undefined && config.test?.randomize !== true) {
			warnings.push({
				kind: "unknown_key", // The doc doesn't specify a "kind" for this warning, but using unknown_key for consistency or we could add cross_field_warning
				filePath,
				line: null,
				fieldPath: "test.seed",
				message: "test.seed has no effect without test.randomize: true",
			});
		}

		// 2. run.shell = "system" with target !== "server-first"
		if (config.run?.shell === "system" && config.target !== "server-first") {
			errors.push({
				kind: "cross_field_error",
				filePath,
				line: null,
				fieldPath: "run.shell",
				message: 'value "system" requires target = "server-first"',
			});
		}

		// 3. useContainers = false with target = "local-first"
		if (config.useContainers === false && config.target === "local-first") {
			errors.push({
				kind: "cross_field_error",
				filePath,
				line: null,
				fieldPath: "useContainers",
				message: 'cannot be false when target = "local-first"',
			});
		}

		// 4. shellCapabilityImages entry references a capability not in shellCapabilities
		if (config.shellCapabilityImages && config.shellCapabilities) {
			for (const key in config.shellCapabilityImages) {
				if (!config.shellCapabilities.includes(key)) {
					errors.push({
						kind: "cross_field_error",
						filePath,
						line: null,
						fieldPath: "shellCapabilityImages",
						message: `key "${key}" not found in shellCapabilities`,
					});
				}
			}
		}

		// 5. disableWasmEdge is not a boolean -> handled by Zod
	}
}
