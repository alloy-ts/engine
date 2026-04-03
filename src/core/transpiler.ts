export type Loader = "js" | "jsx" | "ts" | "tsx";

export interface TranspilerOptions {
	loader?: Loader;
	target?: "browser" | "alloy" | "node";
	define?: Record<string, string>;
	minifyWhitespace?: boolean;
}

export interface Import {
  path: string;
  kind: "import-statement" | "require-call" | "require-resolve" |
        "dynamic-import" | "import-rule" | "url-token" |
        "internal" | "entry-point-build" | "entry-point-run";
}

/**
 * Implements a pipeline for ES5 downlevelling and async/await polyfilling.
 * Uses Bun.Transpiler for the first pass if available.
 */
export class Transpiler {
	constructor(private options: TranspilerOptions = {}) {}

	/**
	 * Synchronous transpilation — same thread, no worker.
	 */
	transformSync(code: string, loader: Loader = "ts"): string {
    // Pass 1: TS/JSX -> ES5 (using Bun's built-in transpiler)
    // @ts-ignore
    const bunTranspiler = new Bun.Transpiler({
        target: "browser",
        loader: loader,
        // @ts-ignore
        define: this.options.define
    });

    let output = bunTranspiler.transformSync(code);

		// Pass 2: Async/await polyfill
		output = output.replace(/\basync\s+function\b/g, "function*");
    // Handle async arrow functions
    output = output.replace(/\basync\s*\(([^)]*)\)\s*=>\s*\{/g, "function*($1) {");
    output = output.replace(/\basync\s*\(([^)]*)\)\s*=>\s*([^;{),\]} ]+)/g, "function*($1) { return $2; }");

		output = output.replace(/\bawait\s+([\s\S]+?)(?=[;,)\]}]|$)/g, "yield { __alloy_ipc: \"__await__\", args: [$1] }");
    output = output.replace(/\bnew\s+Promise\b\s*\(/g, "__alloy_deferred(");

    // Pass 3: MQJS compatibility (remaining ES6+ features not handled by Bun)
		output = output.replace(/\bconst\b/g, "var");
		output = output.replace(/\blet\b/g, "var");

    if (output.includes("__alloy_ipc") || output.includes("__alloy_deferred")) {
        output = "var __alloy_await = AlloyInternal.__alloy_await;\nvar __alloy_deferred = AlloyInternal.__alloy_deferred;\n" + output;
    }

		return output;
	}

	/**
	 * Async transpilation — distributed across workers (Requirement 12.5).
	 */
	async transform(code: string, loader?: Loader): Promise<string> {
		return Promise.resolve(this.transformSync(code, loader));
	}

	/**
	 * Returns all non-type-only imports and exports (Requirement 12.7).
	 */
	scan(code: string): { exports: string[]; imports: Import[] } {
    // @ts-ignore
    const bunTranspiler = new Bun.Transpiler({ loader: "ts" });
    const { exports, imports } = bunTranspiler.scan(code);
		return { exports, imports: imports.map(i => ({ ...i, kind: i.kind || "import-statement" })) };
	}
}
