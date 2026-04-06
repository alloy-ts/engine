export function useConfig(options: {
	configs: Record<string, { default: any }>;
}) {
	// The plugin name should ideally be automatically detected,
	// but for now we'll rely on an internal identifier or just the configs.

	// Register the schema via the internal binding
	const pluginName = (globalThis as any).Alloy?.currentPluginName || "default";
	(globalThis as any).AlloyInternal.bridgeCall("__alloy_config_register", {
		pluginName,
		schema: options,
	});

	return {
		get: (key: string) => {
			return (globalThis as any).AlloyInternal.bridgeCall("__alloy_config_get", {
				key: `plugins.${pluginName}.${key}`,
			});
		},
		set: (key: string, value: any, options?: { persist?: boolean }) => {
			return (globalThis as any).AlloyInternal.bridgeCall("__alloy_config_set", {
				key: `plugins.${pluginName}.${key}`,
				value,
				options,
			});
		},
	};
}
