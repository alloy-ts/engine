// src/core/plugins/config/use_config.ts

declare const AlloyInternal: {
	useConfig(pluginName: string, schema: string): any;
	configSet(key: string, value: any, options?: { persist?: boolean }): void;
};

export function useConfig(pluginName: string, schema: any) {
	const accessorData = AlloyInternal.useConfig(
		pluginName,
		JSON.stringify(schema),
	);

	const accessor = { ...accessorData };

	// Define set method
	Object.defineProperty(accessor, "set", {
		value: (key: string, value: any, options?: { persist?: boolean }) => {
			AlloyInternal.configSet(key, value, options);
			accessor[key] = value; // Update in-memory for the accessor
		},
		enumerable: false,
	});

	return accessor;
}
