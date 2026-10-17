export function main(): string {
  return "Hello, world!";
}

export {
  Compiler,
  Context,
  ContextBuilder,
  Global,
  JsContext,
  JsEngine as Engine,
  JsEngine,
  JsLoader as Loader,
  JsLoader,
  JsWeakEngine as WeakEngine,
  JsWeakEngine,
  Parser,
  QjsContext,
  Script,
  realmEval,
  realmGlobalKeys,
  realmRegisterGlobalProperty,
  realmRunJobs,
} from "../dist/index.js";
