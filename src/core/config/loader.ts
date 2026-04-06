import { AlloyConfig, BUILTIN_DEFAULTS } from "./config";
import { readFile } from "node:fs/promises";

export class ConfigLoader {
    load(options: { cliFlags?: Partial<AlloyConfig> } = {}): AlloyConfig {
        /* discovery -> parse -> merge -> interpolate -> validate -> inject defaults */
        const config = { ...BUILTIN_DEFAULTS, ...options.cliFlags };
        return config;
    }
}
