export function main(): string {
  return "Hello, world!";
}

export {
  add,
  Atom,
  Compiler,
  Context,
  ContextBuilder,
  JsEngine as Engine,
  JsWeakEngine as WeakEngine,
  Global,
  JsLoader as Loader,
  Parser,
  PredefinedAtom,
  Realm,
  Script,
  realmEval,
  realmGlobalKeys,
  realmRegisterGlobalProperty,
  realmRunJobs,
} from "../dist/index.js";
