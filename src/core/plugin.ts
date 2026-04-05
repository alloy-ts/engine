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

export interface SystemPlugin extends AlloyPlugin {
    isSystem: true;
}
