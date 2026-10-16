export function main(): string {
  return "Hello, world!";
}

export {
  BuiltinModuleLoader,
  BuiltinModuleResolver,
  Context,
  Engine,
  FileModuleResolver,
  Global,
  Parser,
  Runtime,
  ScriptModuleLoader,
  add,
} from "../dist/index.js";
