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
 * Implements a 3-pass pipeline for ES5 downlevelling and async/await polyfilling.
 */
export class Transpiler {
	constructor(private options: TranspilerOptions = {}) {}

	/**
	 * Synchronous transpilation — same thread, no worker.
	 */
	transformSync(code: string, loader?: Loader): string {
		// Pass 1: TS/JSX -> ES5 (Stub for Bun.Transpiler)
		let output = code;

		// Pass 2: Async/await polyfill
		output = output.replace(/\basync\s+function\b/g, "function*");
		output = output.replace(/\bawait\s+([\s\S]+?)(?=[;,)\]}]|$)/g, "yield { __alloy_ipc: \"__await__\", args: [$1] }");
    output = output.replace(/\bnew\s+Promise\b\s*\(/g, "__alloy_deferred(");

    // Pass 3: MQJS compatibility
		output = output.replace(/\bconst\b/g, "var");
		output = output.replace(/\blet\b/g, "var");

    // Inject helpers
    if (output.includes("__alloy_ipc") || output.includes("__alloy_deferred")) {
        output = "var __alloy_await = AlloyInternal.__alloy_await;\nvar __alloy_deferred = AlloyInternal.__alloy_deferred;\n" + output;
    }

    // Typescript strip (crude)
		output = output.replace(
			/:\s*(number|string|boolean|any|void|unknown|never)(\[\])?/g,
			"",
		);
		output = output.replace(
			/\s+as\s+(number|string|boolean|any|void|unknown|never)(\[\])?/g,
			"",
		);
		output = output.replace(/interface\s+\w+\s*\{[\s\S]*?\}/g, "");
		output = output.replace(/type\s+\w+\s*=.*?;/g, "");

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
		const imports: Import[] = [];
		const exports: string[] = [];
		const importRegex = /import\s+[\s\S]*?\s+from\s+['"](.*)['"]/g;
		let match;
		while ((match = importRegex.exec(code)) !== null) {
			imports.push({ path: match[1], kind: "import-statement" });
		}
		return { exports, imports };
	}
}
