import { createRequire } from "module";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const require = createRequire(import.meta.url);

const native = require(join(__dirname, "../dist/index.js"));

export const add = native.add;
export const Parser = native.Parser;
export const Engine = native.Engine;
export const Global = native.Global;
export const QuickContextWrapper = native.QuickContextWrapper;
export const QuickLoader = native.QuickLoader;
export const Compiler = native.Compiler;

export function main() {
  return "Hello, world!";
}
