import { AlloyConfigSchema } from "./schema";
import type { AlloyConfig } from "./config";

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
    const result: ValidationResult = { errors: [], warnings: [], valid: true };
    const parsed = AlloyConfigSchema.safeParse(raw);

    if (!parsed.success) {
      for (const issue of parsed.error.issues) {
        result.errors.push({
          kind: "type_error",
          filePath,
          line: null,
          fieldPath: issue.path.join("."),
          message: issue.message,
        });
      }
      result.valid = false;
    }

    const config = raw as Partial<AlloyConfig>;
    if (config.test?.seed !== undefined && !config.test?.randomize) {
      result.warnings.push({
        kind: "unknown_key",
        filePath,
        line: null,
        fieldPath: "test.seed",
        message: "test.seed has no effect without test.randomize: true",
      });
    }

    if (config.run?.shell === "system" && config.target !== "server-first") {
      result.errors.push({
        kind: "cross_field_error",
        filePath,
        line: null,
        fieldPath: "run.shell",
        message: 'value "system" requires target = "server-first"',
      });
      result.valid = false;
    }

    if (config.useContainers === false && config.target === "local-first") {
        result.errors.push({
            kind: "cross_field_error",
            filePath,
            line: null,
            fieldPath: "useContainers",
            message: 'cannot be false when target = "local-first"',
        });
        result.valid = false;
    }

    if (config.shellCapabilityImages) {
        for (const key in config.shellCapabilityImages) {
            if (!config.shellCapabilities?.includes(key)) {
                result.errors.push({
                    kind: "cross_field_error",
                    filePath,
                    line: null,
                    fieldPath: "shellCapabilityImages",
                    message: `key "${key}" not found in shellCapabilities`,
                });
                result.valid = false;
            }
        }
    }

    return result;
  }
}
