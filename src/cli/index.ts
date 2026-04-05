#!/usr/bin/env bun
import { Command } from "commander";
import { evaluate, startRepl, Transpiler, Alloy } from "../main";
import { ensureAlloyHost } from "../core/host_manager";
import { existsSync, mkdirSync, writeFileSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { watch } from "node:fs";
import { BinaryManager } from "../core/engine";
import { spawn } from "node:child_process";
import { ConfigLoader, ConfigValidator } from "../core/config";

const program = new Command();

program
	.name("alloy")
	.description(
		"AlloyScript CLI - Professional industry-standard dual-engine JS toolchain.",
	)
	.version("0.1.0")
    .option("-c, --config <path>", "Path to alloyconfig.toml");

function loadConfig() {
    const opts = program.opts();
    const config = ConfigLoader.load(process.cwd(), opts.config);
    const { errors, warnings } = ConfigValidator.validate(config);

    for (const warn of warnings) console.warn(`Warning: ${warn}`);
    if (errors.length > 0) {
        for (const err of errors) console.error(`Error: ${err}`);
        process.exit(1);
    }
    return config;
}

program
	.command("repl")
	.description("Start the AlloyScript interactive REPL")
	.action(async () => {
        const config = loadConfig();
		const binaryManager = new BinaryManager();
		try {
			const binaryPath = await binaryManager.ensureBinary("0.1.0");
			console.log(`Launching AlloyScript REPL via ${binaryPath}...`);

            // Example override: pass logLevel if configured
            const args = ["repl"];
            if (config.logLevel) args.push("--log-level", config.logLevel);

			const child = spawn(binaryPath, args, { stdio: "inherit" });
			child.on("exit", (code) => process.exit(code || 0));
		} catch (e: any) {
			console.error(`Error: ${e.message}`);
			process.exit(1);
		}
	});

program
	.command("init")
	.description("Initialise a new AlloyScript project")
	.argument("[directory]", "directory to initialise", ".")
	.action((directory) => {
		const targetDir = join(process.cwd(), directory);
		if (!existsSync(targetDir)) {
			mkdirSync(targetDir, { recursive: true });
		}
    const srcDir = join(targetDir, "src");
    if (!existsSync(srcDir)) mkdirSync(srcDir, { recursive: true });

		writeFileSync(
			join(targetDir, "package.json"),
			JSON.stringify(
				{
					name: "my-alloy-app",
					version: "1.0.0",
					dependencies: { "@alloyscript/engine": "*" },
					devDependencies: { "@alloyscript/cli": "*" },
				},
				null,
				2,
			),
		);
    writeFileSync(
			join(targetDir, "tsconfig.json"),
			JSON.stringify(
				{
          "compilerOptions": {
            "target": "ES2022",
            "module": "ESNext",
            "moduleResolution": "bundler",
            "strict": true,
            "jsx": "react-jsx"
          }
				},
				null,
				2,
			),
		);
		writeFileSync(
			join(srcDir, "main.ts"),
			'console.log("Hello from AlloyScript!");\n',
		);

		console.log(`🚀 Project initialised in ${targetDir}`);
	});

program
	.command("build")
	.description("Build and transpile the AlloyScript application")
	.argument("[entrypoint]", "Source TypeScript file", "src/main.ts")
	.option("-o, --outdir <dir>", "Output directory", "dist")
  .option("--target <target>", "Target (browser|alloy|node)")
  .option("--format <format>", "Format (esm|cjs|iife)", "esm")
  .option("--minify", "Minify output", false)
  .option("--sourcemap <type>", "Sourcemap (none|inline|external)", "none")
  .option("--splitting", "Enable splitting", false)
	.action(async (entrypoint, options) => {
        const config = loadConfig();
		console.log(`Building ${entrypoint}...`);

    const buildResult = await Alloy.build({
        entrypoints: [entrypoint],
        outdir: options.outdir || config.install.globalBinDir, // Simplified
        target: (options.target || config.target) as any,
        format: options.format as any,
        minify: options.minify,
        sourcemap: options.sourcemap as any,
        splitting: options.splitting,
        define: config.define,
        // @ts-ignore
        jsx: config.jsx,
        jsxFactory: config.jsxFactory,
        jsxFragment: config.jsxFragment,
        jsxImportSource: config.jsxImportSource
    });

    if (buildResult.success) {
        for (const output of buildResult.outputs) {
            console.log(`Created ${output.path} (${output.size} bytes)`);
        }
    } else {
        for (const log of buildResult.logs) {
            console.error(`[${log.level}] ${log.message}`);
        }
        process.exit(1);
    }
	});

program
	.command("dev")
	.description("Run the project in development mode with hot reloading")
	.argument("[file]", "Entry file", "src/main.ts")
	.action(async (file) => {
		await ensureAlloyHost();
		console.log(`🛠️ Starting AlloyScript development runtime for ${file}...`);

		const run = async () => {
			const code = readFileSync(file, "utf-8");
			try {
				const result = await evaluate(code);
				console.log(result);
			} catch (e) {
				console.error(e);
			}
		};

		watch(file, async (event) => {
			if (event === "change") {
				console.log(`File changed, re-running ${file}...`);
				await run();
			}
		});

		await run();
	});

program
    .command("run", { isDefault: true })
    .description("Execute an AlloyScript or shell file")
    .argument("<file>", "File to execute")
    .action(async (file) => {
        if (file.endsWith('.sh')) {
            const src = readFileSync(file, 'utf-8');
            // @ts-ignore
            const result = await (globalThis as any).$ `${src}`.nothrow();
            process.exit(result.exitCode);
        } else {
            const code = readFileSync(file, 'utf-8');
            await evaluate(code);
        }
    });

program.parse();
