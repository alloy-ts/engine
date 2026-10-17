export function main(): string {
  return "Hello, world!";
}

export {
  add,
  Atom,
  Compiler,
  JsContext as Context,
  JsContextBuilder as ContextBuilder,
  JsEngine as Engine,
  Global,
  Loader,
  Parser,
  PredefinedAtom,
  JsWeakEngine as WeakEngine,
} from "../dist/index.js";
