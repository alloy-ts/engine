import { expect, test, describe, beforeEach } from "bun:test";
import { ConfigManager } from "../config_manager";
import { ConfigPlugin } from "../plugins/config/config_plugin";
import { PluginRegistry } from "./plugins";

describe("ConfigManager and useConfig integration", () => {
	let configManager: ConfigManager;
	let pluginRegistry: PluginRegistry;

	beforeEach(() => {
		configManager = new ConfigManager({
			target: "local-first",
			logLevel: "warn",
			plugins: [],
		});
		pluginRegistry = new PluginRegistry(configManager.getConfig());
	});

	test("ConfigManager can get and set values", () => {
		configManager.set("test.key", "value");
		expect(configManager.get("test.key")).toBe("value");
	});

	test("ConfigManager can register schemas and provide defaults", () => {
		configManager.registerSchema("my-plugin", {
			configs: {
				dbPath: { default: "./data/app.db" },
			},
		});

		expect(configManager.get("plugins.my-plugin.dbPath")).toBe("./data/app.db");
	});

	test("ConfigPlugin registers bindings", () => {
		const plugin = new ConfigPlugin(configManager);
		const engine = {
			bind_fn: (name: string, fn: Function) => {
				(engine as any)[name] = fn;
			},
		};

		plugin.onInit(engine);

		expect((engine as any).__alloy_config_get).toBeDefined();
		expect((engine as any).__alloy_config_set).toBeDefined();
		expect((engine as any).__alloy_config_register).toBeDefined();

		(engine as any).__alloy_config_set({
			key: "test.binding",
			value: "ok",
		});
		expect(configManager.get("test.binding")).toBe("ok");
	});
});
