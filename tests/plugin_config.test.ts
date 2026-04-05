import { expect, test, describe } from "bun:test";
import { PluginManager, ConfigLoader } from "../src/core/config";
import type { AlloyPlugin, PluginContext } from "../src/core/plugin";

describe("Plugin Config System", () => {
    test("Plugins can register and use their own config", async () => {
        const config = ConfigLoader.load();
        const manager = new PluginManager(config);

        class MyPlugin implements AlloyPlugin {
            name = "my-plugin";
            config: any;

            async setup(context: PluginContext) {
                this.config = context.useConfig({
                    configs: {
                        "myPluginValue": { default: "hello" }
                    }
                });
            }
        }

        const plugin = new MyPlugin();
        await manager.register(plugin);

        expect(plugin.config.get("myPluginValue")).toBe("hello");

        plugin.config.set("myPluginValue", "world");
        expect(plugin.config.get("myPluginValue")).toBe("world");

        const finalConfig = manager.getFinalConfig();
        expect(finalConfig.myPluginValue).toBe("world");
    });
});
