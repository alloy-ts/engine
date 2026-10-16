import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const native = require("../dist/index.js");

export const add = native.add;
export const Engine = native.Engine;
export const Parser = native.Parser;
export const Global = native.Global;
export const Context = native.Context;
export const Compiler = native.Compiler;

export function main() {
  return "Hello, world!";
}
