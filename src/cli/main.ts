import { existsSync } from "node:fs";
import { parseArgs } from "node:util";
import { compile } from "./compile";
import { evaluate } from "./evaluate";
import { generate } from "./generate";
import { transpile } from "./transpile";

export async function runCli() {
	const { values, positionals } = parseArgs({
		args: Bun.argv.slice(2),
		options: {
			outdir:      { type: "string",  short: "o" },
			out:         { type: "string" },
			dialect:     { type: "string",  default: "sqlite" },
			watch:       { type: "boolean", short: "w", default: false },
			web:         { type: "boolean", default: true },
			"build-type":{ type: "string",  default: "release" },
			help:        { type: "boolean", short: "h" },
		},
		allowPositionals: true,
	});

	function showHelp() {
		console.log(`
Usage: alloy <command> [options] [arguments]

Commands:
  init [projectname]               Initialise a new AlloyScript project
  repl                             Start the AlloyScript interactive REPL
  compile  <input> [-o <outdir>]   Compile a TS file to a native binary
             [--config <alloyconfig.toml>]
             [--build-type debug]  Debug build (default: release)
  transpile <input> [--out <file>] Transpile a TS file to ES5 for AlloyScript
             [--watch]             Re-transpile on file change
  evaluate <input>                 Evaluate AlloyScript files and print metadata
  generate <input> [-o <outdir>]   Generate TypeScript types and SQL migrations

Options:
  -o, --outdir <dir>       Output directory (compile / generate)
      --out    <file>      Output file path (transpile)
  -w, --watch              Watch mode (transpile)
      --no-web             Disable alloy:web (compile)
      --build-type <type>  release | debug (compile, default: release)
      --dialect <d>        Database dialect (default: sqlite)
  -h, --help               Show this help message
`);
	}

	if (values.help) {
		showHelp();
		process.exit(0);
	}

	if (positionals.length === 0) {
		const defaultPath = "src/data/schemas";
		if (existsSync(defaultPath) || existsSync(`${defaultPath}/index.ts`)) {
			await evaluate(defaultPath);
			process.exit(0);
		}
		showHelp();
		process.exit(0);
	}

	const command  = positionals[0]!;
	const input    = positionals[1];

	const needsInput = ["compile", "transpile", "evaluate", "generate"];
	if (!input && needsInput.includes(command)) {
		console.error(`Error: missing input file for command "${command}"`);
		showHelp();
		process.exit(1);
	}

	switch (command) {
		case "compile":
			await compile(input!, values.outdir ?? "./dist", {
				web:       values.web,
				buildType: (values["build-type"] as "release" | "debug") ?? "release",
			});
			break;

		case "transpile":
			await transpile({
				input: input!,
				out:   values.out,
				watch: values.watch,
			});
			break;

		case "evaluate":
			await evaluate(input!);
			break;

		case "generate":
			await generate(input!, values.dialect as string, values.outdir);
			break;

		default:
			console.error(`Unknown command: ${command}`);
			showHelp();
			process.exit(1);
	}
}
