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
		(globalThis as any).Alloy = {};
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
			return {
				get: (key: string) =>
					(globalThis as any).AlloyInternal.bridgeCall("__alloy_config_get", {
						key: `plugins.${pluginName}.${key}`,
					}),
				set: (key: string, value: any, opts: any) =>
					(globalThis as any).AlloyInternal.bridgeCall("__alloy_config_set", {
						key: `plugins.${pluginName}.${key}`,
						value,
						options: opts,
					}),
			};
		};

		pluginRegistry.register("pluginA", pA);
		pluginRegistry.register("pluginB", pB);

		// Trigger initialization
		await pluginRegistry.load(mockEngine);

		// Verify isolation and defaults
		expect(pA.config.get("keyA")).toBe("valA");
		expect(pB.config.get("keyB")).toBe("valB");
		expect(pA.config.get("keyB")).toBeUndefined();

		// Verify updates
		pA.config.set("keyA", "newValA");
		expect(pA.config.get("keyA")).toBe("newValA");
		expect(configManager.get("plugins.pluginA.keyA")).toBe("newValA");
	});
});
