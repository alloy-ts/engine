import { Alloy } from "./runtime";

export class AlloyTranspiler extends Alloy.Transpiler {
	transformSync(code: string): string {
		return code.replace(/const|let/g, "var");
	}

	bind_global(name: string, value: any): void {
		(globalThis as any).AlloyInternal.bindGlobal(name, value);
	}
}

export async function evaluate(code: string): Promise<any> {
	const transpiler = new AlloyTranspiler();
	try {
		const transformed = transpiler.transformSync(code);
		return (globalThis as any).AlloyInternal.secureEval(transformed);
	} catch (e: any) {
		throw { error: "transpile_failed", message: e.message };
	}
}

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
			console.error(e);
		}
	}

	rl.close();
	(globalThis as any).AlloyInternal.destroy();
}
