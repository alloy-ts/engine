// src/cli/build_pipeline.ts
import type { BuildPlugin } from "../core/plugin";

export interface BuildContext {
    filename: string;
    debug: boolean;
    target: "native" | "browser";
}

export const BUILD_PLUGINS: BuildPlugin[] = [];

export async function runBuildPipeline(source: string, ctx: BuildContext): Promise<string> {
    let output = source;
    // Apply plugins
    for (const plugin of BUILD_PLUGINS) {
        // if (plugin.before_bundle) output = await plugin.before_bundle(output, ctx);
    }
    return output;
}
