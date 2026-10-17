export function main(): string {
  return "Hello, world!";
}

export {
  Context,
  ContextBuilder,
  Engine,
  Global,
  Script,
  realmEval,
  realmGlobalKeys,
  realmGlobalObject,
  realmGlobalThis,
  realmRegisterProperty,
  realmRunJobs,
} from "../dist/index.js";

export type { Realm } from "../dist/index.js";
