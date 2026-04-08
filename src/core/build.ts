// src/core/build.ts
import { Transpiler, Loader } from "./transpiler";
import * as fs from "node:fs";
import * as path from "node:path";

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

/**
 * Requirement 5.1: Programmatic build API
 */
export async function build(options: BuildOptions): Promise<BuildOutput> {
	const outdir = options.outdir || "./dist";
	const outputs: BuildArtifact[] = [];
	const logs: BuildMessage[] = [];
	const transpiler = new Transpiler({ define: options.define });

	try {
		if (!fs.existsSync(outdir)) {
			fs.mkdirSync(outdir, { recursive: true });
		}

		for (const entrypoint of options.entrypoints) {
			const content = fs.readFileSync(entrypoint, "utf-8");
			const transformed = transpiler.transformSync(content, "ts");
			const outPath = path.join(outdir, path.basename(entrypoint, ".ts") + ".js");

			fs.writeFileSync(outPath, transformed);

			outputs.push({
				path: outPath,
				kind: "entry-point",
				size: Buffer.byteLength(transformed),
				async text() { return transformed; },
				async arrayBuffer() { return Buffer.from(transformed).buffer; }
			});
		}

		return { outputs, success: true, logs };
	} catch (e: any) {
		logs.push({ level: "error", message: e.message });
		return { outputs, success: false, logs };
	}
}
