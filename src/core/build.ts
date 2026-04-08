import * as fs from "node:fs/promises";
import * as path from "node:path";
import { AlloyTranspiler, type Loader } from "./transpiler";

export interface BuildOptions {
	entrypoints: string[];
	outdir?: string;
	target?: "browser" | "alloy" | "node";
	format?: "esm" | "cjs" | "iife";
	minify?:
		| boolean
		| { whitespace?: boolean; identifiers?: boolean; syntax?: boolean };
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
		cb: (args: { path: string; importer: string }) => { path: string } | null,
	): void;
	onLoad(
		filter: { filter: RegExp },
		cb: (args: {
			path: string;
		}) => { contents: string; loader?: Loader } | null,
	): void;
}

/**
 * Requirement 13.1: Implement programmatic Alloy.build API
 */
export async function build(options: BuildOptions): Promise<BuildOutput> {
	const transpiler = new AlloyTranspiler({
		target: options.target,
		define: options.define,
		minifyWhitespace:
			typeof options.minify === "boolean"
				? options.minify
				: options.minify?.whitespace,
	});

	const outputs: BuildArtifact[] = [];
	const logs: BuildMessage[] = [];
	const outdir = options.outdir || "./dist";

	try {
		for (const entry of options.entrypoints) {
			const code = await fs.readFile(entry, "utf-8");
			const transformed = transpiler.transformSync(code, "ts");
			const outPath = path.join(
				outdir,
				path.basename(entry).replace(/\.ts$/, ".js"),
			);

			await fs.mkdir(outdir, { recursive: true });
			await fs.writeFile(outPath, transformed);

			outputs.push({
				path: outPath,
				kind: "entry-point",
				size: transformed.length,
				text: async () => transformed,
				arrayBuffer: async () => new TextEncoder().encode(transformed).buffer,
			});
		}
		return { outputs, success: true, logs };
	} catch (err: any) {
		logs.push({ level: "error", message: err.message || String(err) });
		return { outputs, success: false, logs };
	}
}
