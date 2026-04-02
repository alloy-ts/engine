#!/usr/bin/env bun
import { Command } from "commander";
import { evaluate, startRepl, Transpiler } from "@alloyscript/engine";
import { ensureAlloyHost } from "../core/host_manager";
import { existsSync, mkdirSync, writeFileSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { watch } from "node:fs";
import { BinaryManager } from "../core/engine";
import { spawn } from "node:child_process";

const program = new Command();

program
	.name("alloy")
	.description(
		"AlloyScript CLI - Professional industry-standard dual-engine JS toolchain.",
	)
	.version("0.1.0");

program
	.command("repl")
	.description("Start the AlloyScript interactive REPL")
	.action(async () => {
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

		const config = {
			name: "my-alloy-app",
			version: "1.0.0",
			entry: "index.ts",
		};

		writeFileSync(
			join(targetDir, "package.json"),
			JSON.stringify(
				{
					name: config.name,
					version: config.version,
					dependencies: { "@alloyscript/engine": "*" },
					devDependencies: { "@alloyscript/cli": "*" },
				},
				null,
				2,
			),
		);
		writeFileSync(
			join(targetDir, "index.ts"),
			'console.log("Hello from AlloyScript!");\n',
		);

		console.log(`🚀 Project initialised in ${targetDir}`);
	});

program
	.command("build")
	.description("Build and transpile the AlloyScript application")
	.argument("<file>", "Source TypeScript file")
	.option("-o, --output <dir>", "Output directory", "dist")
	.action(async (file, options) => {
		console.log(`Building ${file}...`);
		const transpiler = new Transpiler();
		const code = readFileSync(file, "utf-8");
		const transformed = transpiler.transformSync(code);

		const outputDir = join(process.cwd(), options.output);
		if (!existsSync(outputDir)) mkdirSync(outputDir, { recursive: true });

		const outputFile = join(
			outputDir,
			file.replace(".ts", ".js").split("/").pop()!,
		);
		writeFileSync(outputFile, transformed);
		console.log(`Successfully built to ${outputFile}`);
	});

program
	.command("dev")
	.description("Run the project in development mode with hot reloading")
	.argument("[file]", "Entry file", "index.ts")
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

program.parse();
