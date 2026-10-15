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
  Loader,
  Parser,
  PredefinedAtom,
  Realm,
  Script,
  WeakEngine,
  realmEval,
  realmGlobalKeys,
  realmGlobalObject,
  realmGlobalThis,
  realmRegisterProperty,
  realmRunJobs,
} from "../dist/index.js";
