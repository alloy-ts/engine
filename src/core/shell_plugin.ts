import type { AlloyPlugin, PluginContext } from "./plugin";
import { Alloy } from "./runtime";

export class ShellPlugin implements AlloyPlugin {
    name = "alloy:shell";

    async setup(context: PluginContext) {
        const config = context.useConfig({
            configs: {
                "shellCapabilities": { default: [] as string[] },
                "shellCapabilityImages": { default: {} as Record<string, string> }
            }
        });

        // Register the $ tagged template literal or similar capability
        (globalThis as any).$ = async (strings: TemplateStringsArray, ...values: any[]) => {
            const script = strings.reduce((acc, str, i) => acc + str + (values[i] ?? ""), "");
            // In a real implementation, this would call the WASM shell
            // For foundation, we'll proxy to the engine's internal spawn for now
            // or assume shell.wasm is driven by the host.
            return Alloy.spawn("alloy-shell", [script]);
        };
    }
}
