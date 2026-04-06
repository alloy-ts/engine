import { AlloyConfigSchema, PluginConfigSchema } from "./schema";
import type { AlloyConfig } from "./config";
import type { ZodError } from "zod";

export interface ValidationError {
	kind: "type_error" | "cross_field_error";
	filePath: string;
	line: number | null;
	fieldPath: string;
	message: string;
}

export interface ValidationWarning {
	kind: "unknown_key" | "cross_field_warning";
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
	validate(
		raw: Record<string, unknown>,
		filePath: string,
		source?: string,
	): ValidationResult {
		const errors: ValidationError[] = [];
		const warnings: ValidationWarning[] = [];

		const findLine = (fieldPath: string): number | null => {
			if (!source) return null;
			const parts = fieldPath.split(".");
			const lastPart = parts[parts.length - 1];
			const lines = source.split("\n");
			for (let i = 0; i < lines.length; i++) {
				if (lines[i].includes(lastPart)) return i + 1;
			}
			return null;
		};

		// Check for unknown keys (Stage 5 Phase 1)
		this.checkUnknownKeys(raw, AlloyConfigSchema.shape, "", filePath, warnings);

		// Zod validation for types
		const result = AlloyConfigSchema.safeParse(raw);
		if (!result.success) {
			for (const error of result.error.errors) {
				const fieldPath = error.path.join(".");
				errors.push({
					kind: "type_error",
					filePath,
					line: findLine(fieldPath),
					fieldPath,
					message: error.message,
				});
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

			// Special case for plugins.* and pluginConfigs.*
			if (fullPath.startsWith("pluginConfigs.")) {
				// Defer these to Phase 2
				continue;
			}

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
				let shapeToUse: any = null;

				// Dig into Zod's internal shape
				const def = nestedShape?._def;
				if (def?.shape) {
					shapeToUse = def.shape;
				} else if (def?.innerType?._def?.shape) {
					shapeToUse = def.innerType._def.shape;
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
				kind: "cross_field_warning",
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
		if (config.shellCapabilityImages) {
			const caps = config.shellCapabilities || [];
			for (const key in config.shellCapabilityImages) {
				if (!caps.includes(key)) {
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
	}

	validatePluginConfigs(
		pluginConfigs: Record<string, Record<string, unknown>>,
		registeredSchemas: Map<string, any>,
		filePath: string,
	): ValidationResult {
		const errors: ValidationError[] = [];
		const warnings: ValidationWarning[] = [];

		for (const [pluginName, config] of Object.entries(pluginConfigs)) {
			const schema = registeredSchemas.get(pluginName);
			if (!schema) {
				warnings.push({
					kind: "unknown_key",
					filePath,
					line: null,
					fieldPath: `pluginConfigs.${pluginName}`,
					message: `[plugins.${pluginName}] section has no effect — plugin "${pluginName}" is not loaded or has not declared its config`,
				});
				continue;
			}

			for (const key of Object.keys(config)) {
				if (!(key in schema.configs)) {
					warnings.push({
						kind: "unknown_key",
						filePath,
						line: null,
						fieldPath: `pluginConfigs.${pluginName}.${key}`,
						message: `unknown plugin config key "plugins.${pluginName}.${key}" — not declared by plugin "${pluginName}"; ignored`,
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
