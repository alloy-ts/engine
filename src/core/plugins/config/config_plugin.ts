import { RuntimePlugin } from "../../config/plugins";
import { ConfigManager } from "../../config_manager";

export class ConfigPlugin implements RuntimePlugin {
	private configManager: ConfigManager;

	constructor(configManager: ConfigManager) {
		this.configManager = configManager;
	}

	onInit(engine: any) {
		// Register a JS binding to the configuration manager
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
}
