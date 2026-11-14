#!/usr/bin/env node

import { Engine } from "../dist/index.mjs";

const HELP = `Usage: alloy-engine eval <source>
       alloy-engine eval -

Evaluate source with the @alloy-ts/engine runtime and print the completion
value.

  eval <source>  Evaluate the given source string
  eval -         Read source from standard input
  -h, --help     Show this help
`;

/**
 * @param {unknown} value
 * @returns {string}
 */
function formatValue(value) {
  if (Object.is(value, -0)) {
    return "-0";
  }
  return String(value);
}

/**
 * @param {NodeJS.ReadableStream} stdin
 * @returns {Promise<string>}
 */
async function readStdin(stdin) {
  const chunks = [];
  for await (const chunk of stdin) {
    chunks.push(chunk);
  }
  return Buffer.concat(chunks.map((chunk) => Buffer.from(chunk))).toString("utf8");
}

/**
 * @param {string[]} argv
 * @param {{ stdin: NodeJS.ReadableStream, stdout: NodeJS.WritableStream, stderr: NodeJS.WritableStream }} io
 * @returns {Promise<number>}
 */
export async function run(argv, io = process) {
  const args = argv.slice(2);
  if (args.length === 1 && (args[0] === "-h" || args[0] === "--help" || args[0] === "help")) {
    io.stdout.write(HELP);
    return 0;
  }
  if (args[0] !== "eval" || args.length !== 2 || typeof args[1] !== "string") {
    io.stderr.write(HELP);
    return 1;
  }

  const sourceArg = args[1];
  const source = sourceArg === "-" ? await readStdin(io.stdin) : sourceArg;
  const engine = new Engine();
  const context = engine.createContext();
  try {
    const result = context.eval(source);
    io.stdout.write(`${formatValue(result)}\n`);
    return 0;
  } catch (error) {
    const message = error instanceof Error ? `${error.name}: ${error.message}` : String(error);
    io.stderr.write(`${message}\n`);
    return 1;
  } finally {
    context.dispose();
  }
}

const exitCode = await run(process.argv);
process.exit(exitCode);
