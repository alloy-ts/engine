// src/core/config/validator.ts
import { AlloyConfigSchema, PluginConfigEntrySchema } from "./schema";
import type { AlloyConfig, PluginConfigSchema } from "./config";

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
	validate(raw: Record<string, unknown>, filePath: string): ValidationResult {
		const errors: ValidationError[] = [];
		const warnings: ValidationWarning[] = [];

		// Check for unknown keys (Phase 1: defer [plugins.*])
		this.checkUnknownKeys(raw, AlloyConfigSchema.shape, "", filePath, warnings, true);

		// Zod validation for types
		const result = AlloyConfigSchema.safeParse(raw);
		if (!result.success) {
			for (const error of result.error.errors) {
				errors.push({
					kind: "type_error",
					filePath,
					line: null,
					fieldPath: error.path.join("."),
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

	validatePluginConfigs(
		pluginConfigs: Record<string, Record<string, unknown>>,
		registeredSchemas: Map<string, PluginConfigSchema>,
		filePath: string,
	): ValidationResult {
		const errors: ValidationError[] = [];
		const warnings: ValidationWarning[] = [];

		for (const [pluginName, config] of Object.entries(pluginConfigs)) {
			const schema = registeredSchemas.get(pluginName);
			if (!schema) {
				warnings.push({
					kind: "cross_field_warning",
					filePath,
					line: null,
					fieldPath: `plugins.${pluginName}`,
					message: `[plugins.${pluginName}] section has no effect — plugin "${pluginName}" is not loaded`,
				});
				continue;
			}

			for (const [key, value] of Object.entries(config)) {
				const entrySchema = schema.configs[key];
				if (!entrySchema) {
					warnings.push({
						kind: "unknown_key",
						filePath,
						line: null,
						fieldPath: `plugins.${pluginName}.${key}`,
						message: `unknown plugin config key "plugins.${pluginName}.${key}" — not declared by plugin "${pluginName}"; ignored`,
					});
				} else {
					// Optionally validate type of value against entrySchema.default's type
					// but requirement 21.4 only mentions unknown keys.
				}
			}
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
		deferPlugins: boolean,
	) {
		if (typeof obj !== "object" || obj === null || Array.isArray(obj)) return;

		for (const key in obj) {
			const fullPath = prefix ? `${prefix}.${key}` : key;

			if (deferPlugins && fullPath === "pluginConfigs") continue;

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

				if (nestedShape && nestedShape._def) {
					const def = nestedShape._def;
					if (def.shape) {
						shapeToUse = def.shape;
					} else if (def.innerType && def.innerType._def && def.innerType._def.shape) {
						shapeToUse = def.innerType._def.shape;
					} else if (def.schema && def.schema._def && def.schema._def.shape) {
						// For union or similar
						shapeToUse = def.schema._def.shape;
					}
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
						deferPlugins,
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

		// 5. disableWasmEdge non-boolean is handled by Zod
	}
}
