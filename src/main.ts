export function main(): string {
  return "Hello, world!";
}

export {
  Atom,
  Compiler,
  Context,
  Global,
  JsEngine as Engine,
  JsLoader as Loader,
  JsWeakEngine as WeakEngine,
  Parser,
} from "../dist/index.js";
export type { PredefinedAtom } from "../dist/index.js";
