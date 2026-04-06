export function useConfig(options: {
	configs: Record<string, { default: any }>;
}) {
	if (!(globalThis as any).Alloy?.inPluginInit) {
		throw new Error("useConfig() called outside plugin onInit (Requirement 20.1)");
	}
	// Register the schema via the internal binding
	const pluginName = (globalThis as any).Alloy?.currentPluginName || "default";
	(globalThis as any).AlloyInternal.bridgeCall("__alloy_config_register", {
		pluginName,
		schema: options,
	});

	const accessor = {
		set: (key: string, value: any, opts?: { persist?: boolean }) => {
			return (globalThis as any).AlloyInternal.bridgeCall("__alloy_config_set", {
				key: `pluginConfigs.${pluginName}.${key}`,
				value,
				options: opts,
			});
		},
	};

	// Add direct property access via Proxy (Requirement 22.1)
	return new Proxy(accessor, {
		get: (target, prop) => {
			if (prop === "set") return target.set;
			if (typeof prop === "string") {
				return (globalThis as any).AlloyInternal.bridgeCall(
					"__alloy_config_get",
					{
						key: `pluginConfigs.${pluginName}.${prop}`,
					},
				);
			}
			return undefined;
		},
	}) as any;
}
