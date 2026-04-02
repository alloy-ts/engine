import { Alloy } from "./runtime";
import { AlloyTranspiler } from "./transpiler";
import * as readline from "node:readline/promises";
import { stdin as input, stdout as output } from "node:process";

/**
 * Run a code string through the full dual-engine pipeline:
 *   transpile → MQJSEngine → optional WebView BrowserAPI
 * Resolves with the result value, or rejects with a structured error.
 */
export async function evaluate(code: string): Promise<any> {
	const transpiler = new AlloyTranspiler();
	try {
		const transformed = transpiler.transformSync(code);
		return Alloy.secureEval(transformed);
	} catch (e: any) {
		throw { error: "transpile_failed", message: e.message };
	}
}

/**
 * Start an interactive REPL on stdin/stdout.
 * Resolves when the user sends EOF (Ctrl-D) or types `.exit`.
 */
export async function startRepl(): Promise<void> {
	const rl = readline.createInterface({ input, output });
	console.log("AlloyScript Dual-Engine REPL");

	while (true) {
		const line = await rl.question("alloy> ");
		if (line === ".exit") break;

		try {
			const result = await evaluate(line);
			console.log(result);
		} catch (e: any) {
			console.error(JSON.stringify(e));
		}
	}

	rl.close();
	(globalThis as any).AlloyInternal.destroy();
}
