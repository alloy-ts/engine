import { Transpiler, type Loader } from "../core/transpiler";
import { join, extname, basename, dirname } from "node:path";
import { existsSync } from "node:fs";

export interface TranspileOptions {
  /** Input TypeScript/JS file path */
  input: string;
  /** Output file path. Defaults to <input>.es5.js next to the input file. */
  out?: string;
  /** Watch mode: re-transpile on file change */
  watch?: boolean;
  /** Suppress progress output */
  silent?: boolean;
}

function resolveLoader(filePath: string): Loader {
  const ext = extname(filePath);
  if (ext === ".tsx") return "tsx";
  if (ext === ".jsx") return "jsx";
  if (ext === ".ts")  return "ts";
  return "js";
}

function defaultOutPath(inputPath: string): string {
  const dir  = dirname(inputPath);
  const base = basename(inputPath, extname(inputPath));
  return join(dir, `${base}.es5.js`);
}

async function transpileOnce(opts: TranspileOptions): Promise<void> {
  const inputPath  = join(process.cwd(), opts.input);
  const outputPath = join(process.cwd(), opts.out ?? defaultOutPath(opts.input));

  if (!existsSync(inputPath)) {
    throw new Error(`input file not found: ${opts.input}`);
  }

  const source = await Bun.file(inputPath).text();
  const transpiler = new Transpiler(resolveLoader(inputPath));
  const es5 = transpiler.transformSync(source);

  await Bun.write(outputPath, es5);

  if (!opts.silent) {
    console.log(
      `[alloy:transpile] ${opts.input} → ${opts.out ?? defaultOutPath(opts.input)}` +
      `  (${source.length}B → ${es5.length}B)`,
    );
  }
}

export async function transpile(opts: TranspileOptions): Promise<void> {
  await transpileOnce(opts);

  if (opts.watch) {
    const inputPath = join(process.cwd(), opts.input);
    console.log(`[alloy:transpile] watching ${opts.input}…`);

    const watcher = Bun.watch(inputPath);
    for await (const _event of watcher) {
      try {
        await transpileOnce({ ...opts, silent: false });
      } catch (err) {
        console.error(`[alloy:transpile] error: ${err}`);
      }
    }
  }
}
