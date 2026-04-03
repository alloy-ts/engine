import * as readline from "node:readline";
import * as fs from "node:fs";
import * as path from "node:path";
import { Alloy } from "./runtime";

/**
 * Run a code string through the full dual-engine pipeline
 */
export async function evaluate(code: string): Promise<any> {
	try {
        // In a real implementation, this would handle async/await
        // by driving the generator if secureEval returns one.
		const result = Alloy.secureEval(code);
		return result;
	} catch (e: any) {
		throw { error: e.error || "runtime_error", message: e.message };
	}
}

/**
 * Start an interactive REPL on stdin/stdout (Requirement 9.4)
 */
export async function startRepl(): Promise<void> {
    const VERSION = "0.1.0";
	console.log(`Welcome to AlloyScript v${VERSION} (Requirement 9.1)`);

    const historyPath = path.join(process.env.HOME || "", ".alloy_repl_history");
    let history: string[] = [];
    if (fs.existsSync(historyPath)) {
        history = fs.readFileSync(historyPath, "utf-8").split("\n").filter(Boolean).slice(-1000);
    }

	const rl = readline.createInterface({
		input: process.stdin,
		output: process.stdout,
		prompt: "alloy> ",
        history: history,
        historySize: 1000,
        terminal: true
	});

	rl.prompt();

	return new Promise((resolve) => {
        let multiLineBuffer = "";

		rl.on("line", async (line) => {
			const trimmed = line.trim();

            // Dot-commands
			if (trimmed === ".exit") {
				rl.close();
				return;
			}
            if (trimmed === ".help") {
                console.log(".exit     Exit the REPL");
                console.log(".help     Print this help message");
                console.log(".clear    Clear the screen");
                console.log(".history  Print command history");
                rl.prompt();
                return;
            }
            if (trimmed === ".clear") {
                console.clear();
                rl.prompt();
                return;
            }
            if (trimmed === ".history") {
                console.log(history.join("\n"));
                rl.prompt();
                return;
            }

            // Multi-line detection
            multiLineBuffer += line + "\n";
            const openBrackets = (multiLineBuffer.match(/[\{\[\(]/g) || []).length;
            const closedBrackets = (multiLineBuffer.match(/[\}\]\)]/g) || []).length;

            if (openBrackets > closedBrackets) {
                rl.setPrompt("... ");
                rl.prompt();
                return;
            }

			try {
				const result = await evaluate(multiLineBuffer);
				console.log(result);
				(globalThis as any)._ = result;
			} catch (e: any) {
				console.error(e.message);
				(globalThis as any)._error = e;
			}

            multiLineBuffer = "";
            rl.setPrompt("alloy> ");
			rl.prompt();
		});

		rl.on("close", () => {
            fs.writeFileSync(historyPath, rl.history.join("\n"));
			console.log("\nGoodbye!");
			resolve();
		});
	});
}
