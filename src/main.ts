export function main(): string {
  return "Hello, world!";
}

export {
  Atom,
  Compiler,
  Context,
  Engine,
  Global,
  JsLoader as Loader,
  Parser,
  WeakEngine,
} from "../dist/index.js";

export type { PredefinedAtom } from "../dist/index.js";
