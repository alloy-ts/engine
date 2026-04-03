import * as acorn from "acorn";
import { generate } from "astring";
import { walk } from "estree-walker";
import { build, BuildOptions, BuildOutput } from "./build";

export type Loader = "js" | "jsx" | "ts" | "tsx";

export interface TranspilerOptions {
	loader?: Loader;
	target?: "browser" | "alloy" | "node";
	define?: Record<string, string>;
	minifyWhitespace?: boolean;
}

declare const AlloyInternal: {
    parse(code: string): string | null;
    __alloy_await: Function;
    __alloy_deferred: Function;
};

export class Transpiler {
	constructor(private options: TranspilerOptions = {}) {}

	transformSync(code: string, loader: Loader = "ts"): string {
		// Pass 1: TS/JSX -> ES5 (using Bun.Transpiler)
        const transpiler = new Bun.Transpiler({
            loader: loader,
            target: "browser", // Downlevels to ES5
        });
        let output = transpiler.transformSync(code);

        // Pass 2: Async/await -> generator coroutine rewrite
        const ast = acorn.parse(output, { ecmaVersion: 2020, sourceType: "module" });

        walk(ast as any, {
            enter(node: any) {
                if (node.type === "FunctionDeclaration" && node.async) {
                    node.async = false;
                    node.generator = true;
                }
                if (node.type === "AwaitExpression") {
                    node.type = "YieldExpression";
                    node.argument = {
                        type: "ObjectExpression",
                        properties: [
                            {
                                type: "Property",
                                key: { type: "Identifier", name: "__alloy_ipc" },
                                value: { type: "Literal", value: "__await__" },
                                kind: "init"
                            },
                            {
                                type: "Property",
                                key: { type: "Identifier", name: "args" },
                                value: { type: "ArrayExpression", elements: [node.argument] },
                                kind: "init"
                            }
                        ]
                    };
                }
                if (node.type === "NewExpression" && node.callee.name === "Promise") {
                    node.callee.name = "__alloy_deferred";
                }
            }
        });

        output = generate(ast as any);

        // Pass 3: MQJS compatibility (final cleanup)
        output = output.replace(/\bconst\b/g, "var");
        output = output.replace(/\blet\b/g, "var");

        const helpers = "var __alloy_await = AlloyInternal.__alloy_await;\nvar __alloy_deferred = AlloyInternal.__alloy_deferred;\n";
        output = helpers + output;

        // Validation via JS_Parse
        if (typeof AlloyInternal !== 'undefined' && AlloyInternal.parse) {
            const parseError = AlloyInternal.parse(output);
            if (parseError) {
                throw { error: "transpile_failed", message: parseError };
            }
        }

		return output;
	}

	async transform(code: string, loader?: Loader): Promise<string> {
		return Promise.resolve(this.transformSync(code, loader));
	}

	scan(code: string): { exports: string[]; imports: any[] } {
        const transpiler = new Bun.Transpiler({ loader: "ts" });
        return transpiler.scan(code);
	}

    static build(options: BuildOptions): Promise<BuildOutput> {
        return build(options);
    }
}
