// src/core/config/loader.ts
import { join, dirname } from "node:path";
import { existsSync, readFileSync } from "node:fs";
import { parse } from "smol-toml";
import {
  type AlloyConfig,
  BUILTIN_DEFAULTS_LOCAL,
  BUILTIN_DEFAULTS_SERVER
} from "./config";
import { ConfigValidator, type ValidationResult } from "./validator";

interface DiscoveryResult {
  workspacePath: string | null;
  userPath: string | null;
}

interface ParseResult {
  raw: Record<string, unknown>;
  filePath: string;
  content: string;
}

export interface ConfigLoaderOptions {
  cwd?: string;
  configPath?: string;
  cliFlags?: Partial<AlloyConfig>;
  command?: string; // e.g. "run", "test", "install"
}

export class ConfigLoader {
  private validator = new ConfigValidator();

  load(options?: ConfigLoaderOptions): AlloyConfig {
    const cwd = options?.cwd || process.cwd();
    const discovery = this.discover(cwd, options?.configPath);

    let userConfig: Partial<AlloyConfig> = {};
    if (discovery.userPath && options?.command !== "run") {
      userConfig = this.parse(discovery.userPath).raw as Partial<AlloyConfig>;
    }

    let workspaceConfig: Partial<AlloyConfig> = {};
    let workspaceContent = "";
    if (discovery.workspacePath) {
      const p = this.parse(discovery.workspacePath);
      workspaceConfig = p.raw as Partial<AlloyConfig>;
      workspaceContent = p.content;
    }

    const target = (workspaceConfig.target || userConfig.target || options?.cliFlags?.target || "local-first") as any;
    const defaults = target === "local-first" ? BUILTIN_DEFAULTS_LOCAL : BUILTIN_DEFAULTS_SERVER;

    let merged = this.merge(defaults, userConfig);
    merged = this.merge(merged, workspaceConfig);
    merged = this.merge(merged, options?.cliFlags || {});

    merged = this.interpolate(merged);

    const validationResult = this.validator.validate(
        merged as Record<string, unknown>,
        discovery.workspacePath || "alloyconfig.toml",
        workspaceContent
    );
    this.reportValidation(validationResult);

    return merged as AlloyConfig;
  }

  discover(cwd: string, configPath?: string): DiscoveryResult {
    let workspacePath: string | null = null;
    let userPath: string | null = null;

    if (process.env.XDG_CONFIG_HOME) {
      const p = join(process.env.XDG_CONFIG_HOME, ".alloyconfig.toml");
      if (existsSync(p)) userPath = p;
    }
    if (!userPath && process.env.HOME) {
      const p = join(process.env.HOME, ".alloyconfig.toml");
      if (existsSync(p)) userPath = p;
    }

    if (configPath) {
      workspacePath = join(cwd, configPath);
    } else {
      let current = cwd;
      while (true) {
        const p = join(current, "alloyconfig.toml");
        if (existsSync(p)) {
          workspacePath = p;
          break;
        }
        const pkg = join(current, "package.json");
        if (existsSync(pkg)) {
            workspacePath = join(current, "alloyconfig.toml");
            if (!existsSync(workspacePath)) workspacePath = null;
            break;
        }
        const parent = dirname(current);
        if (parent === current) break;
        current = parent;
      }
    }

    return { workspacePath, userPath };
  }

  parse(path: string): ParseResult {
    try {
      const content = readFileSync(path, "utf-8");
      const raw = parse(content) as Record<string, unknown>;
      return { raw, filePath: path, content };
    } catch (e: any) {
      console.error(`error: ${path}: ${e.message}`);
      process.exit(1);
    }
  }

  merge(
    base: Partial<AlloyConfig>,
    layer: Partial<AlloyConfig>
  ): Partial<AlloyConfig> {
    const merged: any = { ...base };

    for (const key in layer) {
      const val = (layer as any)[key];
      if (key === "define") {
        merged.define = { ...merged.define, ...val };
      } else if (key === "install") {
        const oldScopes = merged.install?.scopes || {};
        merged.install = { ...val };
        merged.install.scopes = { ...oldScopes, ...(val.scopes || {}) };
      } else {
        merged[key] = val;
      }
    }

    return merged;
  }

  interpolate(raw: Partial<AlloyConfig>): Partial<AlloyConfig> {
    const interpolateString = (str: string): string => {
      return str.replace(/\$([a-zA-Z0-9_]+)|\${([a-zA-Z0-9_]+)}/g, (_, v1, v2) => {
        const varName = v1 || v2;
        return process.env[varName] || "";
      });
    };

    const traverse = (obj: any): any => {
      if (typeof obj === "string") {
        return interpolateString(obj);
      }
      if (Array.isArray(obj)) {
        return obj.map(traverse);
      }
      if (obj !== null && typeof obj === "object") {
        const result: any = {};
        for (const key in obj) {
          result[key] = traverse(obj[key]);
        }
        return result;
      }
      return obj;
    };

    return traverse(raw);
  }

  private reportValidation(result: ValidationResult) {
    for (const warning of result.warnings) {
      const lineStr = warning.line ? `:${warning.line}` : "";
      console.warn(`[${warning.filePath}${lineStr}] ${warning.message}`);
    }
    if (!result.valid) {
      for (const error of result.errors) {
        const lineStr = error.line ? `:${error.line}` : "";
        console.error(`[${error.filePath}${lineStr}] field "${error.fieldPath}": ${error.message}`);
      }
      process.exit(1);
    }
  }
}
