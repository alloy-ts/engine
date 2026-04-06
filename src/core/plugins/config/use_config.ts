import { AlloyConfigSchema, PluginConfigSchema } from "./schema";

/**
 * JS-side implementation of useConfig() API (Requirement 20.1)
 * available to plugin entry points during on_init only.
 */
export function useConfig<T extends Record<string, { default: unknown }>>(
	pluginName: string,
	schema: { configs: T },
) {
	// 1. Call native bridge to register schema
	const rawConfig = (globalThis as any).AlloyInternal.useConfig(
		pluginName,
		schema,
	);

	// 2. Return live accessor object (Requirement 22.1)
	const accessor: any = {
		set(entryName: string, value: any, options: { persist?: boolean } = {}) {
			return (globalThis as any).AlloyInternal.configSet(
				pluginName,
				entryName,
				value,
				options.persist || false,
			);
		},
	};

	// Bind getters for each declared entry
	for (const key of Object.keys(schema.configs)) {
		Object.defineProperty(accessor, key, {
			get() {
				// Access values from current in-memory snapshot
				const plugins = (globalThis as any).Alloy.config.plugins;
				if (plugins && plugins[pluginName] && key in plugins[pluginName]) {
					return plugins[pluginName][key];
				}
				return schema.configs[key].default;
			},
			enumerable: true,
		});
	}

	return accessor;
}
