export function main(): string {
  return "Hello, world!";
}

export {
  add,
  Atom,
  Compiler,
  Context,
  Engine,
  Global,
  JsWeakEngine as WeakEngine,
  Loader,
  Parser,
  type PredefinedAtom,
} from "../dist/index.js";
