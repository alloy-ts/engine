import { RuntimePlugin } from "../../config/plugins";
import { ConfigManager } from "../../config_manager";

export class ConfigPlugin implements RuntimePlugin {
	private configManager: ConfigManager;

	constructor(configManager: ConfigManager) {
		this.configManager = configManager;
	}

	onInit(engine: any) {
		// Serialize AlloyConfig to JSON and inject as Alloy.config
		const config = this.configManager.getConfig();
		const configJson = JSON.stringify(config);

		if (engine && engine.eval_fn) {
			// Deep freeze utility (Requirement 22.4)
			engine.eval_fn(`
				(function() {
					function deepFreeze(obj) {
						var propNames = Object.getOwnPropertyNames(obj);
						propNames.forEach(function(name) {
							var prop = obj[name];
							if (typeof prop === 'object' && prop !== null && !Object.isFrozen(prop)) {
								deepFreeze(prop);
							}
						});
						return Object.freeze(obj);
					}
					globalThis.Alloy.config = deepFreeze(${configJson});
				})();
			`);
		}

		// Register JS bindings to the configuration manager
		if (engine && engine.bind_fn) {
			engine.bind_fn("__alloy_config_get", (args: { key: string }) => {
				return this.configManager.get(args.key);
			});

			engine.bind_fn(
				"__alloy_config_set",
				(args: { key: string; value: any; options?: any }) => {
					this.configManager.set(args.key, args.value, args.options);
				},
			);

			engine.bind_fn(
				"__alloy_config_register",
				(args: { pluginName: string; schema: any }) => {
					this.configManager.registerSchema(args.pluginName, args.schema);
				},
			);
		}
	}

	onEvalBegin(engine: any) {
		// Snapshot current in-memory config state into Alloy.config
		const config = this.configManager.getConfig();
		const configJson = JSON.stringify(config);

		if (engine && engine.eval_fn) {
			engine.eval_fn(`
				(function() {
					function deepFreeze(obj) {
						var propNames = Object.getOwnPropertyNames(obj);
						propNames.forEach(function(name) {
							var prop = obj[name];
							if (typeof prop === 'object' && prop !== null && !Object.isFrozen(prop)) {
								deepFreeze(prop);
							}
						});
						return Object.freeze(obj);
					}
					globalThis.Alloy.config = deepFreeze(Object.assign({}, globalThis.Alloy.config, ${configJson}));
				})();
			`);
		}
	}
}
