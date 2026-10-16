export function main(): string {
  return "Hello, world!";
}

export {
  BuiltinLoader,
  BuiltinResolver,
  Context,
  Engine,
  FileResolver,
  Global,
  ModuleLoader,
  NativeLoader,
  Parser,
  Runtime,
  ScriptLoader,
  WeakRuntime,
  add,
} from "../dist/index.js";
