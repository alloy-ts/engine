export type LogLevel = "debug" | "warn" | "error";
export type Target = "local-first" | "server-first";

export interface AlloyConfig {
    target?: Target;
    logLevel?: LogLevel;
    define?: Record<string, string>;
    shellCapabilities?: string[];
    pluginConfigs?: Record<string, any>;
}

export const BUILTIN_DEFAULTS: AlloyConfig = {
    target: "local-first",
    logLevel: "warn",
    define: {},
    shellCapabilities: [],
    pluginConfigs: {}
};
