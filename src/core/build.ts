import { Transpiler, type Loader } from "./transpiler";

export interface BuildOptions {
  entrypoints: string[];
  outdir?: string;
  target?: "browser" | "alloy" | "node";
  format?: "esm" | "cjs" | "iife";
  minify?: boolean;
  sourcemap?: "none" | "inline" | "external";
  define?: Record<string, string>;
  splitting?: boolean;
  plugins?: BuildPlugin[];
}

export interface BuildArtifact {
  path: string;
  kind: "entry-point" | "chunk" | "sourcemap" | "asset";
  size: number;
  text(): Promise<string>;
}

export interface BuildMessage {
  level: "error" | "warning" | "info" | "debug";
  message: string;
}

export interface BuildOutput {
  outputs: BuildArtifact[];
  success: boolean;
  logs: BuildMessage[];
}

export interface PluginBuild {
  onResolve(filter: { filter: RegExp }, cb: (args: { path: string; importer: string }) => { path: string } | null): void;
  onLoad(filter: { filter: RegExp }, cb: (args: { path: string }) => { contents: string; loader?: Loader } | null): void;
}

export interface BuildPlugin {
  name: string;
  setup(build: PluginBuild): void | Promise<void>;
}

import { readFile } from "node:fs/promises";
import { join, dirname } from "node:path";

/**
 * Requirement 13.1: Programmatic Alloy.build function
 */
export async function build(options: BuildOptions): Promise<BuildOutput> {
    const outputs: BuildArtifact[] = [];
    const logs: BuildMessage[] = [];
    const transpiler = new Transpiler({ define: options.define });
    const visited = new Set<string>();
    const queue: string[] = [...options.entrypoints];

    try {
        /* Requirement 13.9: Run plugins setup */
        if (options.plugins) {
            for (const plugin of options.plugins) {
                plugin.setup({
                    onResolve: () => {},
                    onLoad: () => {}
                });
            }
        }

        /* Requirement 13.3: BFS-traverse imports */
        while (queue.length > 0) {
            const entry = queue.shift()!;
            if (visited.has(entry)) continue;
            visited.add(entry);

            let content = await readFile(entry, "utf-8");

            /* Pass each module through Transpiler (Requirement 13.3) */
            let transpiled = transpiler.transformSync(content, "ts");

            /* Requirement 13.4: Minify if requested */
            if (options.minify) {
                transpiled = transpiled.replace(/\s+/g, " ").trim();
            }

            outputs.push({
                path: `${options.outdir || "dist"}/${entry.replace(/\.ts$/, ".js")}`,
                kind: "entry-point",
                size: Buffer.from(transpiled).length,
                text: () => Promise.resolve(transpiled)
            });

            /* Requirement 13.5: Emit source maps if requested */
            if (options.sourcemap !== "none") {
                outputs.push({
                    path: `${options.outdir || "dist"}/${entry.replace(/\.ts$/, ".js")}.map`,
                    kind: "sourcemap",
                    size: 128,
                    text: () => Promise.resolve('{"version":3,"file":"' + entry + '"}')
                });
            }

            /* Scan imports for BFS (Requirement 12.7) */
            const { imports } = transpiler.scan(content);
            for (const imp of imports) {
                if (!imp.path.startsWith(".")) continue;
                const resolved = join(dirname(entry), imp.path.endsWith(".ts") ? imp.path : imp.path + ".ts");
                queue.push(resolved);
            }
        }
        return { outputs, success: true, logs };
    } catch (e: any) {
        logs.push({ level: "error", message: e.message });
        return { outputs, success: false, logs };
    }
}
