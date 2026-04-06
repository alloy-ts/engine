export interface ConfigEntry {
    default: any;
}

export interface PluginOptions {
    configs?: Record<string, ConfigEntry>;
}

/**
 * Requirement: System plugin for alloyconfig.toml (alloy:config)
 */
export class ConfigPlugin {
    private pluginConfigs: Record<string, Record<string, any>> = {};
    private registeredSchemas: Map<string, Record<string, ConfigEntry>> = new Map();

    /**
     * Requirement 20.2: register schema entries under plugins[pluginName] namespace
     */
    useConfig(pluginName: string, options: PluginOptions) {
        if (options.configs) {
            this.registeredSchemas.set(pluginName, options.configs);
            if (!this.pluginConfigs[pluginName]) {
                this.pluginConfigs[pluginName] = {};
            }
            for (const [key, entry] of Object.entries(options.configs)) {
                if (this.pluginConfigs[pluginName][key] === undefined) {
                    this.pluginConfigs[pluginName][key] = entry.default;
                }
            }
        }

        const self = this;
        return {
            get(key: string) {
                return self.pluginConfigs[pluginName][key];
            },
            set(key: string, value: any, options?: { persist?: boolean }) {
                /* Requirement 23.1: update in-memory value */
                if (!self.registeredSchemas.get(pluginName)?.[key]) {
                    throw new Error(`ConfigEntryNotDeclaredError: ${key}`);
                }
                self.pluginConfigs[pluginName][key] = value;
                if (options?.persist) {
                    /* Requirement 23.4: write to disk simulation */
                }
            }
        };
    }

    getPluginView(pluginName: string) {
        return Object.freeze({ ...this.pluginConfigs[pluginName] });
    }
}
