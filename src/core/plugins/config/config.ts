// alloy:config system plugin (JS layer)

function getAlloy() {
	if (!globalThis.Alloy) {
		globalThis.Alloy = { config: {} };
	}
	return globalThis.Alloy;
}

/**
 * Ergonomic API for standard plugin config management.
 *
 * @param {Object} configSpec - { configs: { key: { default: any } } }
 * @param {Object} pluginOptions - { name: string }
 * @returns {Proxy} A proxy object for reading and writing plugin configuration.
 */
export function useConfig(configSpec, pluginOptions) {
	const { configs: defaultConfigs } = configSpec;
	const pluginName = pluginOptions?.name;
	if (!pluginName) throw new Error("pluginOptions.name is required for useConfig");

	const Alloy = getAlloy();

	if (!Alloy.config[pluginName]) {
		Alloy.config[pluginName] = {};
	}

	// Register plugin config sections in the global config object.
	for (const [key, spec] of Object.entries(defaultConfigs)) {
		if (Alloy.config[pluginName][key] === undefined) {
			Alloy.config[pluginName][key] = spec.default;
		}
	}

	return new Proxy(Alloy.config[pluginName], {
		get(target, prop) {
			return target[prop];
		},
		set(target, prop, value) {
			target[prop] = value;
			if (globalThis.__alloy_config_write) {
				globalThis.__alloy_config_write(JSON.stringify([`${pluginName}.${String(prop)}`, value]));
			}
			return true;
		},
	});
}

export function updateConfig(pluginName, key, value) {
	const Alloy = getAlloy();
	if (!Alloy.config[pluginName]) {
		Alloy.config[pluginName] = {};
	}
	Alloy.config[pluginName][key] = value;
	if (globalThis.__alloy_config_write) {
		globalThis.__alloy_config_write(JSON.stringify([`${pluginName}.${key}`, value]));
	}
}

export const config = getAlloy().config;

export default {
	useConfig,
	updateConfig,
	config,
};
