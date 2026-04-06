import { describe, it, expect } from "bun:test";
import { ConfigLoader } from "../src/core/config/loader";
import { PluginManager } from "../src/core/config";
import { AlloyConfig } from "../src/core/config/config";

describe("Plugin Config System", () => {
    it("Plugins can register and use their own config", async () => {
        const initialConfig = new ConfigLoader().load();
        const manager = new PluginManager(initialConfig);

        const plugin = {
            name: "test-plugin",
            setup: async (context: any) => {
                const config = context.useConfig({
                    configs: {
                        myPluginKey: { default: "hello-plugin" }
                    }
                });

                expect(config.get("myPluginKey")).toBe("hello-plugin");
                config.set("myPluginKey", "updated-plugin-value");
                expect(config.get("myPluginKey")).toBe("updated-plugin-value");
            }
        };

        await manager.register(plugin);
        const finalConfig = manager.getFinalConfig();
        expect((finalConfig as any).myPluginKey).toBe("updated-plugin-value");
    });
});
