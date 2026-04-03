import { Transpiler, Loader } from "./transpiler";
import * as fs from "node:fs";
import * as path from "node:path";

export interface BuildOptions {
  entrypoints: string[];
  outdir?: string;                            // default: "./dist"
  target?: "browser" | "alloy" | "node";     // default: "alloy"
  format?: "esm" | "cjs" | "iife";           // default: "esm"
  minify?: boolean | { whitespace?: boolean; identifiers?: boolean; syntax?: boolean };
  sourcemap?: "none" | "inline" | "external"; // default: "none"
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
  const outdir = options.outdir || "./dist";
  if (!fs.existsSync(outdir)) fs.mkdirSync(outdir, { recursive: true });

  const outputs: BuildArtifact[] = [];
  const logs: BuildMessage[] = [];
  const transpiler = new Transpiler();

  for (const entry of options.entrypoints) {
    try {
      const fullPath = path.resolve(entry);
      const content = fs.readFileSync(fullPath, "utf-8");
      const loader = path.extname(entry).slice(1) as Loader;
      const transformed = transpiler.transformSync(content, loader);

      const outPath = path.join(outdir, path.basename(entry, path.extname(entry)) + ".js");
      fs.writeFileSync(outPath, transformed);

      outputs.push({
        path: outPath,
        kind: "entry-point",
        size: transformed.length,
        text: () => Promise.resolve(transformed),
        arrayBuffer: () => Promise.resolve(new TextEncoder().encode(transformed).buffer)
      });
    } catch (e: any) {
      logs.push({ level: "error", message: e.message || String(e) });
    }
  }

  return {
    outputs,
    success: logs.length === 0,
    logs
  };
}
