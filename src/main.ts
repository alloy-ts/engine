export function main(): string {
  return "Hello, world!";
}

export {
  BuiltinModuleLoader,
  BuiltinModuleResolver,
  Compiler,
  Context,
  Engine,
  FileModuleResolver,
  Global,
  Parser,
  ScriptModuleLoader,
  add,
} from "../dist/index.js";
