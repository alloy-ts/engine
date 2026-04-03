import * as readline from "node:readline";
import { Alloy } from "./runtime";

/**
 * Run a code string through the full dual-engine pipeline
 */
export async function evaluate(code: string): Promise<any> {
	try {
		const result = Alloy.secureEval(code);
    if (result && result.error === "transpile_failed") {
        throw result;
    }
		return result;
	} catch (e: any) {
		throw { error: e.error || "runtime_error", message: e.message };
	}
}

/**
 * Start an interactive REPL on stdin/stdout (Requirement 9.4)
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
      if (trimmed === ".help") {
          console.log(".exit, .help, .clear");
          rl.prompt();
          return;
      }

			try {
				const result = await evaluate(trimmed);
				console.log(result);
				(globalThis as any)._ = result;
			} catch (e: any) {
				console.error(e.message || e);
				(globalThis as any)._error = e;
			}
			rl.prompt();
		});

		rl.on("close", () => {
			console.log("\nGoodbye!");
			resolve();
		});
	});
}
