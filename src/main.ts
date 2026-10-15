// Native bindings re-exported from dist/index.js
// @ts-ignore
import * as native from "../dist/index.js";

export const add: (left: number, right: number) => number = native.add;
export const Parser: any = native.Parser;
export const Engine: any = native.Engine;
export const JsEngine: any = native.Engine;
export const Compiler: any = native.Compiler;
export const Global: any = native.Global;
export const OxcParser: any = native.OxcParser;

export function main(): string {
  return "Hello, world!";
}
