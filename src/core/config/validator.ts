// src/core/config/validator.ts
import { AlloyConfigSchema } from "./schema";
import type { AlloyConfig } from "./config";
import { parse } from "smol-toml";

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
  validate(raw: Record<string, unknown>, filePath: string, content?: string): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    const lineMap = content ? this.buildLineMap(content) : new Map<string, number>();

    // 1. Check for unknown keys (Warnings)
    this.checkUnknownKeys(raw, AlloyConfigSchema.shape, "", filePath, warnings, lineMap);

    // 2. Zod Schema Validation (Type Errors)
    const result = AlloyConfigSchema.safeParse(raw);
    if (!result.success) {
      for (const issue of result.error.issues) {
        const fieldPath = issue.path.join(".");
        errors.push({
          kind: "type_error",
          filePath,
          line: lineMap.get(fieldPath) || null,
          fieldPath,
          message: issue.message,
        });
      }
    }

    // 3. Cross-field validation
    if (result.success) {
      const config = result.data as AlloyConfig;
      this.validateCrossFieldRules(config, filePath, errors, warnings, lineMap);
    }

    return {
      errors,
      warnings,
      valid: errors.length === 0,
    };
  }

  private buildLineMap(content: string): Map<string, number> {
    const lineMap = new Map<string, number>();
    const lines = content.split("\n");

    // Very simple heuristic to find keys in TOML
    let currentSection = "";
    for (let i = 0; i < lines.length; i++) {
        const line = lines[i].trim();
        if (line.startsWith("[") && line.endsWith("]")) {
            currentSection = line.slice(1, -1).trim();
            lineMap.set(currentSection, i + 1);
        } else if (line.includes("=")) {
            const key = line.split("=")[0].trim();
            const fullKey = currentSection ? `${currentSection}.${key}` : key;
            lineMap.set(fullKey, i + 1);
        }
    }
    return lineMap;
  }

  private checkUnknownKeys(
    raw: any,
    shape: any,
    prefix: string,
    filePath: string,
    warnings: ValidationWarning[],
    lineMap: Map<string, number>
  ) {
    if (!raw || typeof raw !== "object" || Array.isArray(raw)) return;

    for (const key in raw) {
      const fieldPath = prefix ? `${prefix}.${key}` : key;
      if (!(key in shape)) {
        warnings.push({
          kind: "unknown_key",
          filePath,
          line: lineMap.get(fieldPath) || null,
          fieldPath,
          message: `unknown key "${fieldPath}" — ignored`,
        });
      } else {
        const subShape = shape[key];
        let innerShape = subShape;
        while (innerShape && innerShape._def && (innerShape._def.typeName === "ZodOptional" || innerShape._def.typeName === "ZodDefault")) {
            innerShape = innerShape._def.innerType;
        }

        if (innerShape && innerShape._def && innerShape._def.typeName === "ZodObject") {
          this.checkUnknownKeys(raw[key], innerShape.shape, fieldPath, filePath, warnings, lineMap);
        }
      }
    }
  }

  private validateCrossFieldRules(
    config: AlloyConfig,
    filePath: string,
    errors: ValidationError[],
    warnings: ValidationWarning[],
    lineMap: Map<string, number>
  ) {
    const target = config.target || "local-first";

    if (config.test?.seed !== undefined && config.test?.randomize !== true) {
      warnings.push({
        kind: "cross_field_warning",
        filePath,
        line: lineMap.get("test.seed") || null,
        fieldPath: "test.seed",
        message: 'test.seed has no effect without test.randomize: true',
      });
    }

    if (config.run?.shell === "system" && target !== "server-first") {
      errors.push({
        kind: "cross_field_error",
        filePath,
        line: lineMap.get("run.shell") || null,
        fieldPath: "run.shell",
        message: 'value "system" requires target = "server-first"',
      });
    }

    if (config.useContainers === false && target === "local-first") {
      errors.push({
        kind: "cross_field_error",
        filePath,
        line: lineMap.get("useContainers") || null,
        fieldPath: "useContainers",
        message: 'cannot be false when target = "local-first"',
      });
    }

    if (config.shellCapabilityImages && config.shellCapabilities) {
      for (const cap in config.shellCapabilityImages) {
        if (!config.shellCapabilities.includes(cap)) {
          errors.push({
            kind: "cross_field_error",
            filePath,
            line: lineMap.get("shellCapabilityImages") || null,
            fieldPath: "shellCapabilityImages",
            message: `key "${cap}" not found in shellCapabilities`,
          });
        }
      }
    }
  }
}
