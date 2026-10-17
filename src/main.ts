export function main(): string {
  return "Hello, world!";
}

export {
  Context,
  Engine,
  Global,
  Parser,
  Realm,
  Script,
  realmEval,
  realmGlobalKeys,
  realmGlobalObject,
  realmGlobalThis,
  realmRegisterProperty,
  realmRunJobs,
} from "../dist/index.js";
