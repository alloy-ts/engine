import { parse } from 'acorn';
import { generate } from 'astring';

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

import { build, type BuildOptions, type BuildOutput } from "./build";

/**
 * Implements a pipeline for ES5 downlevelling and async/await polyfilling.
 * Uses Bun.Transpiler for the first pass and acorn/astring for AST transformations.
 */
export class Transpiler {
	static async build(options: BuildOptions): Promise<BuildOutput> {
		return build(options);
	}

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

    // Pass 2: Robust Async/await polyfill via AST
    const ast = parse(output, { ecmaVersion: 'latest', sourceType: 'module' });

    const transformNode = (node: any) => {
        if (!node) return;

        if (node.type === 'FunctionDeclaration' && node.async) {
            node.async = false;
            node.generator = true;
        } else if (node.type === 'FunctionExpression' && node.async) {
            node.async = false;
            node.generator = true;
        } else if (node.type === 'ArrowFunctionExpression' && node.async) {
            // Lower arrow async to function* expression
            node.type = 'FunctionExpression';
            node.async = false;
            node.generator = true;
        } else if (node.type === 'AwaitExpression') {
            node.type = 'YieldExpression';
            node.argument = {
                type: 'ObjectExpression',
                properties: [
                    {
                        type: 'Property',
                        key: { type: 'Identifier', name: '__alloy_ipc' },
                        value: { type: 'Literal', value: '__await__' },
                        kind: 'init'
                    },
                    {
                        type: 'Property',
                        key: { type: 'Identifier', name: 'args' },
                        value: { type: 'ArrayExpression', elements: [node.argument] },
                        kind: 'init'
                    }
                ]
            };
        } else if (node.type === 'NewExpression' && node.callee.name === 'Promise') {
             node.callee.name = '__alloy_deferred';
        }

        for (const key in node) {
            const child = node[key];
            if (Array.isArray(child)) {
                child.forEach(transformNode);
            } else if (child && typeof child === 'object' && child.type) {
                transformNode(child);
            }
        }
    };

    transformNode(ast);
    output = generate(ast);

    // Pass 3: MQJS compatibility
		output = output.replace(/\bconst\b/g, "var");
		output = output.replace(/\blet\b/g, "var");
    output = output.replace(/\bmeta\.env\.NODE_ENV\b/g, "\"production\"");
    output = output.replace(/\bmeta\.file\b/g, "\"<source>\"");

    const internal = (globalThis as any).AlloyInternal;
    if (typeof internal !== "undefined" && internal.parse) {
        const parseError = internal.parse(output);
        if (parseError) throw { error: "transpile_failed", message: parseError };
    }

		return output;
	}

	async transform(code: string, loader?: Loader): Promise<string> {
		return Promise.resolve(this.transformSync(code, loader));
	}

	scan(code: string): { exports: string[]; imports: Import[] } {
    // @ts-ignore
    const bunTranspiler = new Bun.Transpiler({ loader: "ts" });
    const { exports, imports } = bunTranspiler.scan(code);
		return { exports, imports: imports.map(i => ({ ...i, kind: i.kind || "import-statement" })) };
	}
}
