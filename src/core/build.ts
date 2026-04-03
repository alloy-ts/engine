import { Transpiler, type Loader } from "./transpiler";

export interface BuildOptions {
  entrypoints: string[];
  outdir?: string;
  target?: "browser" | "alloy" | "node";
  format?: "esm" | "cjs" | "iife";
  minify?: boolean | { whitespace?: boolean; identifiers?: boolean; syntax?: boolean };
  sourcemap?: "none" | "inline" | "external";
  define?: Record<string, string>;
  external?: string[];
  splitting?: boolean;
  plugins?: BuildPlugin[];
}

export interface BuildArtifact {
  path: string;
  kind: "entry-point" | "chunk" | "sourcemap" | "asset";
  size: number;
  text(): Promise<string>;
  arrayBuffer(): Promise<ArrayBuffer>;
}

export interface BuildMessage {
  level: "error" | "warning" | "info" | "debug";
  message: string;
  position?: { file: string; line: number; column: number };
}

export interface BuildOutput {
  outputs: BuildArtifact[];
  success: boolean;
  logs: BuildMessage[];
}

export interface BuildPlugin {
  name: string;
  setup(build: PluginBuild): void | Promise<void>;
}

export interface PluginBuild {
  onResolve(
    filter: { filter: RegExp },
    cb: (args: { path: string; importer: string }) => { path: string } | null
  ): void;
  onLoad(
    filter: { filter: RegExp },
    cb: (args: { path: string }) => { contents: string; loader?: Loader } | null
  ): void;
}

export async function build(options: BuildOptions): Promise<BuildOutput> {
    const outputs: BuildArtifact[] = [];
    const logs: BuildMessage[] = [];
    const transpiler = new Transpiler();

    for (const entry of options.entrypoints) {
        try {
            // Simplified build logic for MVP
            const code = ""; // Load from entry
            const transformed = transpiler.transformSync(code);
            outputs.push({
                path: `${options.outdir || "./dist"}/${entry.replace(/\.tsx?$/, ".js")}`,
                kind: "entry-point",
                size: transformed.length,
                text: async () => transformed,
                arrayBuffer: async () => new TextEncoder().encode(transformed).buffer
            });
        } catch (e: any) {
            logs.push({ level: "error", message: e.message });
        }
    }

    return {
        outputs,
        success: logs.every(l => l.level !== "error"),
        logs
    };
}
