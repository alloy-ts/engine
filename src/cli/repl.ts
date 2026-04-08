import { spawn } from "node:child_process";
import { BinaryManager } from "../core/engine";

const binaryManager = new BinaryManager();
try {
	const binaryPath = await binaryManager.ensureBinary("0.1.0");
	console.log(`Launching AlloyScript REPL via ${binaryPath}...`);

	const child = spawn(binaryPath, ["repl"], { stdio: "inherit" });
	child.on("exit", (code) => process.exit(code || 0));
} catch (e: any) {
	console.error(`Error: ${e.message}`);
	process.exit(1);
}
