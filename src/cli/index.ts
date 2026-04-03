#!/usr/bin/env bun
import { Command } from "commander";
import { evaluate, startRepl, Transpiler } from "@alloyscript/engine";
import { getConfig } from "../core/config";
import { ensureAlloyHost } from "../core/host_manager";
import { existsSync, mkdirSync, writeFileSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { watch } from "node:fs";
import { BinaryManager } from "../core/engine";
import { spawn } from "node:child_process";

const program = new Command();

program
	.name("alloy")
	.option("-c, --config <path>", "path to alloyconfig.toml")
	.description(
		"AlloyScript CLI - Professional industry-standard dual-engine JS toolchain.",
	)
	.version("0.1.0");

program
	.command("repl")
	.description("Start the AlloyScript interactive REPL")
	.action(async (options) => {
		const config = getConfig(program.opts().config);
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
        if (!existsSync(srcDir)) mkdirSync(srcDir);

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
                    compilerOptions: {
                        target: "ES2022",
                        module: "ESNext",
                        moduleResolution: "bundler",
                        strict: true,
                        jsx: "react-jsx"
                    }
                },
                null,
                2
            )
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
	.argument("[entrypoints...]", "Source TypeScript files")
	.option("-o, --outdir <dir>", "Output directory")
    .option("-t, --target <target>", "Target platform (browser|alloy|node)")
    .option("-f, --format <format>", "Output format (esm|cjs|iife)")
    .option("--minify", "Minify output")
    .option("--sourcemap <type>", "Sourcemap type (none|inline|external)")
    .option("--splitting", "Enable code splitting")
	.action(async (entrypoints, options) => {
        const config = getConfig(program.opts().config);
        if (!entrypoints || entrypoints.length === 0) {
            entrypoints = ["src/main.ts"];
        }
		console.log(`Building ${entrypoints.join(", ")}...`);

        const result = await Transpiler.build({
            entrypoints,
            outdir: options.outdir || config.install?.globalDir || "dist", // fallback logic
            target: options.target || config.target || "alloy",
            format: options.format || "esm",
            minify: options.minify !== undefined ? options.minify : false,
            sourcemap: options.sourcemap || "none",
            splitting: options.splitting !== undefined ? options.splitting : false,
            define: config.define,
            loader: config.loader
        });

        if (result.success) {
            for (const artifact of result.outputs) {
                console.log(`  - ${artifact.path} (${artifact.size} bytes)`);
            }
            console.log(`Successfully built to ${options.outdir}`);
        } else {
            for (const log of result.logs) {
                console.error(`[${log.level}] ${log.message}`);
            }
            process.exit(1);
        }
	});

program
	.command("run")
	.description("Run an AlloyScript file or shell script")
	.argument("<file>", "File to run")
    .option("--silent", "suppress echoing of the command string")
	.action(async (file, options) => {
        const config = getConfig(program.opts().config);
        const silent = options.silent !== undefined ? options.silent : config.run?.silent;

		if (file.endsWith(".sh")) {
            if (!silent) console.log(`Running shell script: ${file}`);
			const src = readFileSync(file, "utf-8");
			// Shebang is stripped in lexer
			// @ts-ignore
			const result = await $`${src}`.nothrow();
			process.exit(result.exitCode);
		} else {
            if (!silent) console.log(`Running AlloyScript: ${file}`);
			const code = readFileSync(file, "utf-8");
			try {
				await evaluate(code);
			} catch (e) {
				console.error(e);
				process.exit(1);
			}
		}
	});

program
	.command("dev")
	.description("Run the project in development mode with hot reloading")
	.argument("[file]", "Entry file", "src/main.ts")
	.action(async (file) => {
        const config = getConfig(program.opts().config);
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

program.parse();
