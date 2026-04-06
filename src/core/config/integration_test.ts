import { expect, test, describe, beforeEach } from "bun:test";
import { ConfigManager } from "../config_manager";
import { ConfigPlugin } from "../plugins/config/config_plugin";
import { PluginRegistry, RuntimePlugin } from "./plugins";

describe("AlloyConfig System Plugin Integration", () => {
	let configManager: ConfigManager;
	let pluginRegistry: PluginRegistry;
	let mockEngine: any;

	beforeEach(() => {
		configManager = new ConfigManager({
			target: "local-first",
			logLevel: "warn",
			plugins: [],
		});
		pluginRegistry = new PluginRegistry(configManager.getConfig());

		const bindings: Record<string, Function> = {};
		mockEngine = {
			bind_fn: (name: string, fn: Function) => {
				bindings[name] = fn;
			},
			eval_fn: (code: string) => {
				// Simulate global environment
				if (code.includes("currentPluginName")) {
					const match = code.match(/currentPluginName = "([^"]+)"/);
					if (match) (globalThis as any).Alloy.currentPluginName = match[1];
				}
			},
			bridgeCall: (name: string, args: any) => {
				return bindings[name](args);
			},
		};

		// Setup global Alloy object for simulation
		(globalThis as any).Alloy = { inPluginInit: true };
		(globalThis as any).AlloyInternal = {
			bridgeCall: mockEngine.bridgeCall,
		};

		// Register and initialize the system plugin
		const configPlugin = new ConfigPlugin(configManager);
		pluginRegistry.register("alloy:config", configPlugin);
	});

	test("Multiple plugins can useConfig with isolation", async () => {
		// Mock two plugins
		const pA = {
			config: null as any,
			onInit(engine: any) {
				// @ts-ignore
				this.config = useConfig({
					configs: {
						keyA: { default: "valA" },
					},
				});
			},
		};

		const pB = {
			config: null as any,
			onInit(engine: any) {
				// @ts-ignore
				this.config = useConfig({
					configs: {
						keyB: { default: "valB" },
					},
				});
			},
		};

		// Mock useConfig in the test environment (normally in runtime.ts)
		(globalThis as any).useConfig = (options: any) => {
			const pluginName = (globalThis as any).Alloy.currentPluginName;
			(globalThis as any).AlloyInternal.bridgeCall("__alloy_config_register", {
				pluginName,
				schema: options,
			});
			const accessor = {
				set: (key: string, value: any, opts: any) =>
					(globalThis as any).AlloyInternal.bridgeCall("__alloy_config_set", {
						key: `pluginConfigs.${pluginName}.${key}`,
						value,
						options: opts,
					}),
			};
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
			});
		};

		pluginRegistry.register("pluginA", pA);
		pluginRegistry.register("pluginB", pB);

		// Trigger initialization
		await pluginRegistry.load(mockEngine);

		// Verify isolation and defaults
		expect(pA.config.keyA).toBe("valA");
		expect(pB.config.keyB).toBe("valB");
		expect(pA.config.keyB).toBeUndefined();

		// Verify updates
		pA.config.set("keyA", "newValA");
		expect(pA.config.keyA).toBe("newValA");
		expect(configManager.get("pluginConfigs.pluginA.keyA")).toBe("newValA");
	});

	test("Inline config takes precedence over TOML section", async () => {
		const configLoader = new (require("./loader").ConfigLoader)();
		const rawConfig = {
			plugins: [
				{
					name: "my-plugin",
					config: { dbPath: "./inline.db" },
				},
			],
			pluginConfigs: {
				"my-plugin": { dbPath: "./toml.db" },
			},
		};

		const finalConfig = configLoader.injectDefaults(rawConfig);
		expect(finalConfig.pluginConfigs["my-plugin"].dbPath).toBe("./inline.db");
	});

	test("Schema merging and duplicate warnings", () => {
		const logSpy = console.warn;
		let warned = false;
		console.warn = () => {
			warned = true;
		};

		configManager.registerSchema("my-plugin", {
			configs: { key1: { default: "v1" } },
		});
		configManager.registerSchema("my-plugin", {
			configs: { key1: { default: "v2" }, key2: { default: "v3" } },
		});

		expect(configManager.get("pluginConfigs.my-plugin.key1")).toBe("v1");
		expect(configManager.get("pluginConfigs.my-plugin.key2")).toBe("v3");
		expect(warned).toBe(true);

		console.warn = logSpy;
	});
});
