import { runBuildPipeline } from "./build_pipeline";
import { run, ShellError } from "../core/shell";
import { join, basename, extname, resolve } from "node:path";
import { existsSync, writeFileSync, rmSync, mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import * as babel from "@babel/core";
import alloyCpsPlugin from "../babel-plugin-alloy-cps";

export interface CompileOptions {
  input: string;
  output?: string;
  key?: string;
  noEncrypt?: boolean;
  debug?: boolean;
  web?: boolean;
}

async function cpsTransform(src: string): Promise<string> {
  const result = await babel.transformAsync(src, {
    plugins: [alloyCpsPlugin],
    generatorOpts: {
      compact: false,
      retainLines: false,
    },
  });
  return result?.code || "";
}

export async function compile(opts: CompileOptions): Promise<void> {
  const inputPath = resolve(opts.input);
  const keyPath = opts.key ?? process.env.ALLOY_SIGNING_KEY;

  if (!existsSync(inputPath)) {
    throw new Error(`input file not found: ${opts.input}`);
  }

  const tmpDir = mkdtempSync(join(tmpdir(), "alloyc-"));

  try {
    // Stage 1: Bundling and TS stripping via Bun.build
    console.log(`[alloyc] bundling ${opts.input}…`);
    const stage1Out = join(tmpDir, "stage1.js");
    const buildResult = await Bun.build({
      entrypoints: [inputPath],
      target: "browser",
      minify: false,
    });

    if (!buildResult.success) {
      console.error(`[alloyc] Stage 1 failed:\n${buildResult.logs.map(l => l.message).join("\n")}`);
      process.exit(1);
    }

    const stage1Src = await buildResult.outputs[0].text();
    writeFileSync(stage1Out, stage1Src);

    // Stage 2+3: Build Pipeline (shim + CPS transform)
    const ctx = {
      filename: opts.input,
      debug: opts.debug ?? false,
      target: "desktop" as const,
    };

    console.log(`[alloyc] transforming…`);
    const transformed = await runBuildPipeline(stage1Src, ctx, cpsTransform);

    if (opts.debug) {
      const debugOut = (opts.output ?? opts.input).replace(/\.[^.]+$/, "") + ".debug.js";
      writeFileSync(debugOut, transformed);
      console.log(`[alloyc] debug output → ${debugOut}`);
    }

    const stage3Out = join(tmpDir, "stage3.js");
    writeFileSync(stage3Out, transformed);

    // Stage 4: Bytecode compilation via alloyc_compile
    const outputPath = opts.output ?? (basename(inputPath, extname(inputPath)) + ".alloy");
    const flags = opts.noEncrypt ? ["--no-encrypt"] : [];
    if (keyPath) flags.push("--key", keyPath);

    console.log(`[alloyc] compiling to ${outputPath}…`);
    try {
      // Find alloyc_compile next to the alloy entry point or assume in path
      const alloycCompilePath = "alloyc_compile";
      await run`${alloycCompilePath} -i ${stage3Out} -o ${outputPath} ${flags}`;
    } catch (err: unknown) {
      if (err instanceof ShellError) {
        console.error(`[alloyc] compilation failed:\n${(err as ShellError).result.stderr.trim()}`);
        process.exit(1);
      }
      throw err;
    }

    console.log(`[alloyc] ✓ ${outputPath}`);
  } finally {
    rmSync(tmpDir, { recursive: true, force: true });
  }
}
