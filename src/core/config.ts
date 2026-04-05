export type JSXTransformMode = "react" | "react-jsx" | "react-jsxdev" | "preserve" | "solid";
export type LogLevel = "debug" | "warn" | "error";
export type LoaderType = "jsx" | "js" | "ts" | "tsx" | "css" | "file" | "json" | "toml" | "wasm" | "napi" | "base64" | "dataurl" | "text";
export type AutoInstallBehavior = "auto" | "force" | "disable" | "fallback";
export type LinkerType = "hoisted" | "isolated";
export type RunShell = "alloy" | "system";
export type DeploymentTarget = "local-first" | "server-first";

export interface CoverageThresholdValue {
    line?: number;
    function?: number;
    statement?: number;
}

export interface RegistryConfig {
    url: string;
    token?: string;
    username?: string;
    password?: string;
}

export interface TestConfig {
    root: string;
    preload: string[];
    pathIgnorePatterns: string[];
    smol: boolean;
    coverage: boolean;
    coverageThreshold?: number | CoverageThresholdValue;
    coverageSkipTestFiles: boolean;
    coveragePathIgnorePatterns: string[];
    coverageReporter: string[];
    coverageDir: string;
    randomize: boolean;
    seed?: number;
    rerunEach: number;
    retry: number;
    concurrentTestGlob?: string;
    onlyFailures: boolean;
    reporter: {
        dots: boolean;
        junit?: string;
    };
}

export interface InstallConfig {
    optional: boolean;
    dev: boolean;
    peer: boolean;
    production: boolean;
    exact: boolean;
    saveTextLockfile: boolean;
    auto?: AutoInstallBehavior;
    frozenLockfile: boolean;
    dryRun: boolean;
    globalDir?: string;
    globalBinDir?: string;
    registry?: string | RegistryConfig;
    linkWorkspacePackages: boolean;
    scopes: Record<string, string | RegistryConfig>;
    ca?: string;
    cafile?: string;
    cache: {
        dir?: string;
        disable: boolean;
        disableManifest: boolean;
    };
    lockfile: {
        save: boolean;
        print?: "yarn";
    };
    linker: LinkerType;
    security: {
        scanner?: string;
    };
    minimumReleaseAge?: number;
    minimumReleaseAgeExcludes: string[];
}

export interface RunConfig {
    shell: RunShell;
    alloy: boolean;
    silent: boolean;
}

export interface DebugConfig {
    editor?: string;
}

export interface AlloyConfig {
    preload: string[];
    jsx?: JSXTransformMode;
    jsxFactory?: string;
    jsxFragment?: string;
    jsxImportSource?: string;
    cryptomem: boolean;
    logLevel?: LogLevel;
    define: Record<string, string>;
    loader: Record<string, LoaderType>;
    telemetry: boolean;
    env?: boolean | { file: boolean };
    console: {
        depth: number;
    };
    target: DeploymentTarget;
    shellCapabilities: string[];
    shellCapabilityImages: Record<string, string>;
    disableWasmEdge: boolean;
    useContainers: boolean;
    test: TestConfig;
    install: InstallConfig;
    run: RunConfig;
    debug: DebugConfig;
    [key: string]: any;
}

export const DEFAULT_CONFIG: AlloyConfig = {
    preload: [],
    cryptomem: false,
    define: {},
    loader: {},
    telemetry: true,
    console: {
        depth: 2
    },
    target: "local-first",
    shellCapabilities: [],
    shellCapabilityImages: {},
    disableWasmEdge: false,
    useContainers: true,
    test: {
        root: ".",
        preload: [],
        pathIgnorePatterns: [],
        smol: false,
        coverage: false,
        coverageSkipTestFiles: false,
        coveragePathIgnorePatterns: [],
        coverageReporter: ["text"],
        coverageDir: "coverage",
        randomize: false,
        rerunEach: 1,
        retry: 0,
        onlyFailures: false,
        reporter: {
            dots: false
        }
    },
    install: {
        optional: true,
        dev: true,
        peer: true,
        production: false,
        exact: false,
        saveTextLockfile: true,
        frozenLockfile: false,
        dryRun: false,
        linkWorkspacePackages: true,
        scopes: {},
        cache: {
            disable: false,
            disableManifest: false
        },
        lockfile: {
            save: true
        },
        linker: "hoisted",
        minimumReleaseAgeExcludes: []
    },
    run: {
        shell: "alloy",
        alloy: false,
        silent: false
    },
    debug: {}
};

import { readFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import { homedir } from "node:os";

import type { AlloyPlugin, PluginContext, ConfigController, ConfigOptions } from "./plugin";

export class ConfigPlugin implements AlloyPlugin {
    name = "alloy:config";
    private customConfigs: Map<string, any> = new Map();
    private activeConfig: AlloyConfig;

    constructor(initialConfig: AlloyConfig) {
        this.activeConfig = initialConfig;
    }

    async setup(context: PluginContext) {
        // System config plugin doesn't need to call useConfig on itself
    }

    registerCustomConfig(name: string, definition: any) {
        if (!(name in this.activeConfig)) {
            (this.activeConfig as any)[name] = definition.default;
        }
    }

    get(name: string): any {
        return (this.activeConfig as any)[name];
    }

    set(name: string, value: any) {
        (this.activeConfig as any)[name] = value;
    }

    getMergedConfig(): AlloyConfig {
        return this.activeConfig;
    }
}

export class PluginManager implements PluginContext {
    private plugins: AlloyPlugin[] = [];
    private configPlugin: ConfigPlugin;

    constructor(config: AlloyConfig) {
        this.configPlugin = new ConfigPlugin(config);
    }

    useConfig<T>(options: ConfigOptions<T>): ConfigController<T> {
        const manager = this;
        for (const [key, def] of Object.entries(options.configs)) {
            this.configPlugin.registerCustomConfig(key, def);
        }

        return {
            get(name: string): T {
                return manager.configPlugin.get(name);
            },
            set(name: string, value: T): void {
                manager.configPlugin.set(name, value);
            }
        };
    }

    async register(plugin: AlloyPlugin) {
        this.plugins.push(plugin);
        await plugin.setup(this);
    }

    getFinalConfig(): AlloyConfig {
        return this.configPlugin.getMergedConfig();
    }
}

export class ConfigLoader {
    static interpolate(value: any): any {
        if (typeof value === "string") {
            return value.replace(/\$(\w+)|\${(\w+)}/g, (_, v1, v2) => {
                const varName = v1 || v2;
                return process.env[varName] || "";
            });
        }
        if (Array.isArray(value)) {
            return value.map(v => ConfigLoader.interpolate(v));
        }
        if (value !== null && typeof value === "object") {
            const result: any = {};
            for (const key in value) {
                result[key] = ConfigLoader.interpolate(value[key]);
            }
            return result;
        }
        return value;
    }

    static load(cwd: string = process.cwd(), explicitPath?: string): AlloyConfig {
        let config: any = { ...DEFAULT_CONFIG };

        // 1. Load Global Config
        const globalHome = process.env.XDG_CONFIG_HOME || homedir();
        const globalPath = join(globalHome, ".alloyconfig.toml");
        if (existsSync(globalPath)) {
            try {
                const content = readFileSync(globalPath, "utf-8");
                // @ts-ignore
                const parsed = Bun.TOML.parse(content);
                config = { ...config, ...parsed };
            } catch (e) {
                console.error(`Error parsing global config at ${globalPath}: ${e}`);
            }
        }

        // 2. Load Local Config
        let localPath = explicitPath;
        if (!localPath) {
            const projectRoot = cwd; // Simple for now
            localPath = join(projectRoot, "alloyconfig.toml");
        }

        if (existsSync(localPath)) {
            try {
                const content = readFileSync(localPath, "utf-8");
                // @ts-ignore
                const parsed = Bun.TOML.parse(content);
                // Shallow merge top-level keys
                for (const key in parsed) {
                    if (typeof parsed[key] === "object" && !Array.isArray(parsed[key]) && config[key]) {
                        config[key] = { ...config[key], ...parsed[key] };
                    } else {
                        config[key] = parsed[key];
                    }
                }
            } catch (e) {
                console.error(`Error parsing local config at ${localPath}: ${e}`);
            }
        }

        // 3. Interpolate environment variables
        config = ConfigLoader.interpolate(config);

        // 4. Default target-based isolation settings
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

export class ConfigValidator {
    static validate(config: AlloyConfig): { errors: string[], warnings: string[] } {
        const errors: string[] = [];
        const warnings: string[] = [];

        if (config.target !== "local-first" && config.target !== "server-first") {
            errors.push(`Invalid target: ${config.target}. Must be "local-first" or "server-first".`);
        }

        if (config.target === "local-first" && config.useContainers === false) {
            errors.push(`useContainers cannot be false when target is "local-first".`);
        }

        if (config.target === "local-first" && config.run.shell === "system") {
            errors.push(`run.shell cannot be "system" when target is "local-first".`);
        }

        if (config.test.seed !== undefined && !config.test.randomize) {
            warnings.push(`test.seed has no effect without test.randomize: true.`);
        }

        // Schema type checking - simplified for MVP
        const checkType = (path: string, val: any, expected: string) => {
            if (val !== undefined && typeof val !== expected) {
                errors.push(`Type mismatch at ${path}: expected ${expected}, got ${typeof val}.`);
            }
        };

        checkType("telemetry", config.telemetry, "boolean");
        checkType("cryptomem", config.cryptomem, "boolean");
        checkType("disableWasmEdge", config.disableWasmEdge, "boolean");
        checkType("console.depth", config.console?.depth, "number");

        if (config.define) {
            for (const key in config.define) {
                if (typeof config.define[key] !== "string") {
                    errors.push(`Type mismatch at define.${key}: expected string, got ${typeof config.define[key]}.`);
                }
            }
        }

        if (config.shellCapabilityImages) {
            for (const key in config.shellCapabilityImages) {
                if (!config.shellCapabilities.includes(key)) {
                    errors.push(`Capability "${key}" in shellCapabilityImages is not listed in shellCapabilities.`);
                }
            }
        }

        return { errors, warnings };
    }
}
