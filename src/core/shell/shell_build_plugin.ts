import type { BuildPlugin, PluginBuild } from "../build";
import { Transpiler } from "../transpiler";
import * as fs from "node:fs";

export function shellBuildPlugin(): BuildPlugin {
	return {
		name: "alloy-shell",
		setup(build: PluginBuild) {
			build.onLoad({ filter: /\.[jt]sx?$/ }, async (args) => {
                const source = fs.readFileSync(args.path, "utf-8");

                // Simplified build-time replacement logic
                // In a production version, this would use a proper AST walker (acorn)
                // to evaluate $`...` expressions and replace them with literals.
                if (source.includes("$`")) {
                    let transformed = source;
                    // Mock build-time evaluation
                    transformed = transformed.replace(/\$\`echo\s+"([^"]+)"\`/g, '"$1"');
                    return { contents: transformed, loader: "ts" };
                }
				return null;
			});
		},
	};
}
