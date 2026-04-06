// src/core/plugins/config/use_config.ts
import { PluginConfigSchema } from "../../config/config";

declare global {
	var __alloy_use_config: (pluginName: string, schema: string) => string;
	var __alloy_config_set: (
		pluginName: string,
		key: string,
		value: string,
		persist: boolean,
	) => string;
	var Alloy: {
		config: any;
	};
}

export class ConfigAccessor {
	private pluginName: string;

	constructor(pluginName: string, schema: PluginConfigSchema) {
		this.pluginName = pluginName;
		const initialValues = Alloy.config.plugins?.[pluginName] || {};

		for (const key in schema) {
			const defaultValue = schema[key].default;
			const currentVal = initialValues[key];

			Object.defineProperty(this, key, {
				get: () => Alloy.config.plugins?.[this.pluginName]?.[key] ?? defaultValue,
				enumerable: true,
			});
		}
	}

	set(key: string, value: any, options?: { persist?: boolean }) {
		const res = __alloy_config_set(
			this.pluginName,
			key,
			JSON.stringify(value),
			options?.persist || false,
		);
		const result = JSON.parse(res);
		if (result.error) {
			throw new Error(result.error);
		}
	}
}

export function useConfig(
	pluginName: string,
	schema: PluginConfigSchema,
): ConfigAccessor {
	// Call bridge to register schema
	const res = __alloy_use_config(pluginName, JSON.stringify(schema));
	const result = JSON.parse(res);
	if (result.error) {
		throw new Error(result.error);
	}

	return new ConfigAccessor(pluginName, schema);
}
