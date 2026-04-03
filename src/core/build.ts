import * as fs from "node:fs";
import * as path from "node:path";
import { ConfigLoader } from "./config/loader";
import { shellBuildPlugin } from "./shell/shell_build_plugin";
import { type Loader, Transpiler } from "./transpiler";
import { Linker } from "./linker";

export interface BuildOptions {
	entrypoints: string[];
	outdir?: string; // default: "./dist"
	target?: "browser" | "alloy" | "node"; // default: "alloy"
	format?: "esm" | "cjs" | "iife"; // default: "esm"
	minify?:
		| boolean
		| { whitespace?: boolean; identifiers?: boolean; syntax?: boolean };
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
		cb: (args: { path: string; importer: string }) => { path: string } | null,
	): void;
	onLoad(
		filter: { filter: RegExp },
		cb: (args: {
			path: string;
		}) => { contents: string; loader?: Loader } | null,
	): void;
}

export async function build(options: BuildOptions): Promise<BuildOutput> {
	const configLoader = new ConfigLoader();
	const config = configLoader.load({
		cliFlags: options as any,
	});

	// Add shell build plugin by default
	const plugins = options.plugins || [];
	plugins.push(shellBuildPlugin());
	options.plugins = plugins;

	const outdir = options.outdir || config.install?.globalDir || "./dist";
	if (!fs.existsSync(outdir)) fs.mkdirSync(outdir, { recursive: true });

	const outputs: BuildArtifact[] = [];
	const logs: BuildMessage[] = [];
	const linker = new Linker();

	try {
		const bundle = await linker.link(options.entrypoints, {
			minify: typeof options.minify === "boolean" ? options.minify : !!options.minify,
			treeShaking: true,
		});

		const outPath = path.join(outdir, "bundle.js");
		fs.writeFileSync(outPath, bundle);

		outputs.push({
			path: outPath,
			kind: "entry-point",
			size: bundle.length,
			text: () => Promise.resolve(bundle),
			arrayBuffer: () =>
				Promise.resolve(new TextEncoder().encode(bundle).buffer),
		});
	} catch (e: unknown) {
		const error = e as { message?: string };
		logs.push({ level: "error", message: error.message || String(e) });
	}

	return {
		outputs,
		success: logs.length === 0,
		logs,
	};
}
