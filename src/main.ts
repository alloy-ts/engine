export function main(): string {
  return "Hello, world!";
}

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
  PredefinedAtom,
  Realm,
  Script,
  WeakEngine,
  realmEval,
  realmGlobalKeys,
  realmRegisterGlobalProperty,
  realmRunJobs,
} from "../dist/index.js";
