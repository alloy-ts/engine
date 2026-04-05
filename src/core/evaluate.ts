import * as readline from "node:readline";
import { Transpiler } from "./transpiler";

/**
 * Run a code string through the full dual-engine pipeline
 */
export async function evaluate(code: string): Promise<any> {
	try {
        const transpiler = new Transpiler();
        const transformed = transpiler.transformSync(code);
		// In a real CLI this would send the code to the host binary
        console.log("Transpiled code:", transformed);
		return transformed;
	} catch (e: any) {
		throw { error: e.error || "runtime_error", message: e.message };
	}
}

/**
 * Start an interactive REPL on stdin/stdout
 */
export async function startRepl(): Promise<void> {
	console.log(`Welcome to AlloyScript v0.1.0`);
	const rl = readline.createInterface({
		input: process.stdin,
		output: process.stdout,
		prompt: "alloy> ",
	});

	rl.prompt();

	return new Promise((resolve) => {
		rl.on("line", async (line) => {
			const trimmed = line.trim();
			if (trimmed === ".exit") {
				rl.close();
				return;
			}

			try {
				const result = await evaluate(trimmed);
				console.log(result);
				// @ts-expect-error
				globalThis._ = result;
			} catch (e: any) {
				console.error(e.message);
				// @ts-expect-error
				globalThis._error = e;
			}
			rl.prompt();
		});

		rl.on("close", () => {
			console.log("\nGoodbye!");
			resolve();
		});
	});
}
