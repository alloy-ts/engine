import { Transpiler } from "bun";
import { build, BuildOptions, BuildOutput } from "./build";

export type Loader = "js" | "jsx" | "ts" | "tsx";

export interface TranspilerOptions {
	loader?: Loader;
	target?: "browser" | "alloy" | "node";
	define?: Record<string, string>;
	tsconfig?: string | object;
	macro?: Record<string, string>;
	exports?: { eliminate?: string[]; replace?: Record<string, string> };
	trimUnusedImports?: boolean;
	minifyWhitespace?: boolean;
	inline?: boolean;
}

export interface Import {
	path: string;
	kind:
		| "import-statement"
		| "require-call"
		| "require-resolve"
		| "dynamic-import"
		| "import-rule"
		| "url-token"
		| "internal"
		| "entry-point-build"
		| "entry-point-run";
}

export class AlloyTranspiler {
	private bunTranspiler: any;

	constructor(options?: TranspilerOptions) {
		this.bunTranspiler = new Transpiler({
			loader: options?.loader || "ts",
			target: options?.target || "browser",
			define: options?.define,
			minifyWhitespace: options?.minifyWhitespace,
			inline: options?.inline,
		});
	}

	transformSync(code: string, loader?: Loader): string {
		let output = this.bunTranspiler.transformSync(code, loader);

		if (
			output.includes("async") ||
			output.includes("await") ||
			output.includes("Promise")
		) {
			output = this.rewriteAsync(output);
			output =
				"var __alloy_await = AlloyInternal.__alloy_await;\n" +
				"var __alloy_deferred = AlloyInternal.__alloy_deferred;\n" +
				output;
		}

		output = output.replace(/\bconst\b/g, "var").replace(/\blet\b/g, "var");

		// @ts-ignore
		const parseError =
			typeof AlloyInternal !== "undefined" && AlloyInternal.parse
				? AlloyInternal.parse(output)
				: null;
		if (parseError) {
			throw { error: "transpile_failed", message: parseError };
		}

		return output;
	}

	private rewriteAsync(code: string): string {
		// Use word boundaries for better robustness
		let res = code.replace(/\basync\s+function\b/g, "function* ");
		res = res.replace(
			/\bawait\s+([^;{}]+)/g,
			"yield { __alloy_ipc: '__await__', args: [$1] }",
		);
		res = res.replace(/\bnew\s+Promise\b/g, "__alloy_deferred");
		return res;
	}

	async transform(code: string, loader?: Loader): Promise<string> {
		return this.transformSync(code, loader);
	}

	scan(code: string): { exports: string[]; imports: Import[] } {
		return this.bunTranspiler.scan(code);
	}

	scanImports(code: string): Import[] {
		return this.bunTranspiler.scanImports(code);
	}

	static async build(options: BuildOptions): Promise<BuildOutput> {
		return build(options);
	}
}
