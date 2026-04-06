import { BUILTIN_DEFAULTS } from "./config";
import { ConfigLoader } from "./loader";
import { ConfigValidator } from "./validator";

export type { AlloyConfig } from "./config";

export interface AlloyPlugin {
    name: string;
    version?: string;
    setup(context: PluginContext): void | Promise<void>;
}

export interface PluginContext {
    useConfig<T>(options: ConfigOptions<T>): ConfigController<T>;
}

export interface ConfigOptions<T> {
    configs: {
        [key: string]: {
            default: T;
            description?: string;
        };
    };
}

export interface ConfigController<T> {
    get(name: string): T;
    set(name: string, value: T): void;
}

export class ConfigPlugin implements AlloyPlugin {
    name = "alloy:config";
    private activeConfig: any;

    constructor(initialConfig: any) {
        this.activeConfig = initialConfig;
    }

    async setup(context: PluginContext) {
        // System config plugin doesn't need to call useConfig on itself
    }

    registerCustomConfig(name: string, definition: any) {
        if (!(name in this.activeConfig)) {
            this.activeConfig[name] = definition.default;
        }
    }

    get(name: string): any {
        return this.activeConfig[name];
    }

    set(name: string, value: any) {
        this.activeConfig[name] = value;
    }

    getMergedConfig(): any {
        return this.activeConfig;
    }
}

export class PluginManager implements PluginContext {
    private plugins: AlloyPlugin[] = [];
    private configPlugin: ConfigPlugin;

    constructor(config: any) {
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

    getFinalConfig(): any {
        return this.configPlugin.getMergedConfig();
    }
}
