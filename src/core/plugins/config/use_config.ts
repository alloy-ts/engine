// src/core/plugins/config/use_config.ts
import { PluginConfigSchema } from "../../config/config";

declare const AlloyInternal: {
	__alloy_use_config(pluginName: string, schema: string): string;
	__alloy_config_set(pluginName: string, key: string, value: any, options?: { persist?: boolean }): void;
};

declare const Alloy: {
	currentPluginName: string;
};

export function useConfig<T extends Record<string, { default: unknown }>>(
	schema: { configs: T }
): { [K in keyof T]: T[K]["default"] } & {
	set<K extends keyof T>(
		entryName: K,
		value: T[K]["default"],
		options?: { persist?: boolean }
	): void;
} {
	const pluginName = Alloy.currentPluginName;
	const schemaJson = JSON.stringify(schema);

	// Requirement 20.2: Register schema entries via bridge
	const accessorJson = AlloyInternal.__alloy_use_config(pluginName, schemaJson);
	const initialValues = JSON.parse(accessorJson);

	const accessor = {
		set<K extends keyof T>(
			entryName: K,
			value: T[K]["default"],
			options?: { persist?: boolean }
		) {
			// Requirement 23.1: Update in-memory
			(accessor as any)[entryName] = value;

			// Requirement 23.3: Call bridge for persistent/cross-process updates
			AlloyInternal.__alloy_config_set(pluginName, entryName as string, value, options);
		}
	} as any;

	// Mix in initial values
	for (const key in initialValues) {
		accessor[key] = initialValues[key];
	}

	return accessor;
}
