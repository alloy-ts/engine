import { existsSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { join, dirname } from "node:path";
import * as TOML from "smol-toml";
import type { AlloyConfig } from "./config";
import { BUILTIN_DEFAULTS } from "./config";
import { ConfigValidator } from "./validator";

export interface DiscoveryResult {
  workspacePath: string | null;
  userPath: string | null;
}

export interface ParseResult {
  raw: Record<string, unknown>;
  filePath: string;
}

export interface ConfigLoaderOptions {
  cwd?: string;
  configPath?: string;
  cliFlags?: Partial<AlloyConfig>;
}

export class ConfigLoader {
  load(options: ConfigLoaderOptions = {}): AlloyConfig {
    const cwd = options.cwd || process.cwd();
    const discovery = this.discover(cwd, options.configPath);

    const userRaw = discovery.userPath ? this.parse(discovery.userPath).raw : {};
    const workspaceRaw = discovery.workspacePath ? this.parse(discovery.workspacePath).raw : {};

    // Filter UserConfig for 'alloy run' (Requirement 1.9)
    const isAlloyRun = process.argv.includes("run");
    const effectiveUserRaw = isAlloyRun ? {} : userRaw;

    const merged = this.merge(effectiveUserRaw, workspaceRaw, options.cliFlags);
    const interpolated = this.interpolate(merged);

    const validator = new ConfigValidator();
    const validationResult = validator.validate(interpolated, discovery.workspacePath || "alloyconfig.toml");

    if (!validationResult.valid) {
      for (const err of validationResult.errors) {
        console.error(`[${err.filePath}:${err.line || "?"}] ${err.fieldPath}: ${err.message}`);
      }
      process.exit(1);
    }

    for (const warn of validationResult.warnings) {
        console.warn(`[${warn.filePath}:${warn.line || "?"}] ${warn.fieldPath}: ${warn.message}`);
    }

    return this.injectDefaults(interpolated as Record<string, unknown>);
  }

  discover(cwd: string, configPath?: string): DiscoveryResult {
    let workspacePath: string | null = null;
    if (configPath) {
      workspacePath = join(cwd, configPath);
    } else {
      let curr = cwd;
      while (true) {
        const p = join(curr, "alloyconfig.toml");
        if (existsSync(p)) {
          workspacePath = p;
          break;
        }
        const pkg = join(curr, "package.json");
        if (existsSync(pkg)) break;
        const next = dirname(curr);
        if (next === curr) break;
        curr = next;
      }
    }

    let userPath: string | null = null;
    const xdg = process.env.XDG_CONFIG_HOME;
    if (xdg) {
      const p = join(xdg, ".alloyconfig.toml");
      if (existsSync(p)) userPath = p;
    }
    if (!userPath) {
      const p = join(homedir(), ".alloyconfig.toml");
      if (existsSync(p)) userPath = p;
    }

    return { workspacePath, userPath };
  }

  parse(path: string): ParseResult {
    try {
      const content = readFileSync(path, "utf-8");
      const raw = TOML.parse(content) as Record<string, unknown>;
      return { raw, filePath: path };
    } catch (e: any) {
      console.error(`error: ${path}:${e.line || "?"}:${e.column || "?"}: ${e.message}`);
      process.exit(1);
    }
  }

  merge(
    user: Record<string, unknown>,
    workspace: Record<string, unknown>,
    cliFlags?: Partial<AlloyConfig>
  ): Record<string, unknown> {
    const merged = JSON.parse(JSON.stringify(BUILTIN_DEFAULTS));

    const applyLayer = (layer: Record<string, unknown>) => {
      for (const key in layer) {
        if (key === "define") {
          merged.define = { ...merged.define, ...(layer.define as any) };
        } else if (key === "install" && (layer[key] as any)?.scopes) {
            if (!merged.install) merged.install = {};
            merged.install.scopes = { ...merged.install.scopes, ...(layer.install as any).scopes };
            const { scopes, ...rest } = layer.install as any;
            merged.install = { ...merged.install, ...rest };
        } else if (typeof layer[key] === "object" && layer[key] !== null && !Array.isArray(layer[key]) && merged[key]) {
            merged[key] = { ...merged[key], ...(layer[key] as any) };
        } else {
          merged[key] = layer[key];
        }
      }
    };

    applyLayer(user);
    applyLayer(workspace);
    if (cliFlags) applyLayer(cliFlags as Record<string, unknown>);

    return merged;
  }

  interpolate(raw: any): any {
    if (typeof raw === "string") {
      return raw.replace(/\$(\w+)|\${(\w+)}/g, (_, v1, v2) => {
        const varName = v1 || v2;
        return process.env[varName] || "";
      });
    }
    if (Array.isArray(raw)) {
      return raw.map((v) => this.interpolate(v));
    }
    if (raw !== null && typeof raw === "object") {
      const result: any = {};
      for (const key in raw) {
        result[key] = this.interpolate(raw[key]);
      }
      return result;
    }
    return raw;
  }

  injectDefaults(raw: Record<string, unknown>): AlloyConfig {
    const config = raw as any;
    if (config.target === "server-first") {
      if (config.disableWasmEdge === undefined) config.disableWasmEdge = true;
      if (config.useContainers === undefined) config.useContainers = false;
    } else {
      if (config.disableWasmEdge === undefined) config.disableWasmEdge = false;
      if (config.useContainers === undefined) config.useContainers = true;
    }
    return config as AlloyConfig;
  }
}
