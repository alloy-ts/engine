export function main(): string {
  return "Hello, world!";
}

export {
  Compiler,
  Context,
  Global,
  JsEngine as Engine,
  JsLoader as Loader,
  Parser,
  Script,
  JsWeakEngine as WeakEngine,
  realmEval,
  realmGlobalKeys,
  realmRegisterGlobalProperty,
  realmRunJobs,
} from "../dist/index.js";

export type { Realm } from "../dist/index.js";
