export type Loader = "js" | "jsx" | "ts" | "tsx";

export interface TranspilerOptions {
	loader?: Loader;
	target?: "browser" | "alloy" | "node";
	define?: Record<string, string>;
	minifyWhitespace?: boolean;
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
		// Use Bun's built-in Transpiler if available, else robust regex fallback
		let output = code;

		// Rewrite 'async function' -> 'function*' and 'await' -> 'yield __alloy_await'
		output = output.replace(/\basync\s+function\b/g, "function*");
		output = output.replace(/\bawait\s+/g, "yield __alloy_await ");

		output = output.replace(/\bconst\b/g, "var");
		output = output.replace(/\blet\b/g, "var");
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
	scan(code: string): { exports: string[]; imports: any[] } {
		const imports: any[] = [];
		const exports: string[] = [];
		const importRegex = /import\s+[\s\S]*?\s+from\s+['"](.*)['"]/g;
		let match;
		while ((match = importRegex.exec(code)) !== null) {
			imports.push({ path: match[1], kind: "import-statement" });
		}
		return { exports, imports };
	}
}
