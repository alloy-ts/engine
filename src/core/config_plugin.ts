export interface ConfigEntry {
    default: any;
}

export interface PluginOptions {
    configs?: Record<string, ConfigEntry>;
}

/**
 * Requirement: System plugin for alloyconfig.toml
 */
export class ConfigPlugin {
    private configs: Record<string, any> = {};

    useConfig(options: PluginOptions) {
        if (options.configs) {
            for (const [key, entry] of Object.entries(options.configs)) {
                this.configs[key] = entry.default;
            }
        }
    }

    get(key: string) {
        return this.configs[key];
    }

    set(key: string, value: any) {
        this.configs[key] = value;
    }
}
