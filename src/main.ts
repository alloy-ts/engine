import { Engine as _Engine } from "../dist/index.js";

export function main(): string {
  return "Hello, world!";
}

// Backend selector sentinels, exposed as `Engine.backend.Boa` / `Engine.backend.QuickJs`.
(_Engine as unknown as { backend: unknown }).backend = {
  Boa: "boa",
  QuickJs: "quickjs",
};

export {
  Context,
  Engine,
  Global,
  Realm,
  realmEval,
  realmGlobalKeys,
  realmGlobalObject,
  realmGlobalThis,
  realmRegisterProperty,
  realmRunJobs,
  Script,
} from "../dist/index.js";
