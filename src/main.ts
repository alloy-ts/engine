export function main(): string {
  return "Hello, world!";
}

export type { PredefinedAtom } from "../dist/index.js";
export {
  add,
  Atom,
  Compiler,
  Context,
  ContextBuilder,
  Engine,
  Global,
  Loader,
  Parser,
  WeakEngine,
} from "../dist/index.js";
