export function main(): string {
  return "Hello, world!";
}

export {
  Engine,
  Global,
  Parser,
  Compiler,
  add,
  Context,
  BuiltinResolver,
  BuiltinLoader,
  FileResolver,
  ScriptLoader,
} from "../dist/index.js";
