export function main(): string {
  return "Hello, world!";
}

export {
  Engine,
  Global,
  Parser,
  add,
  Context,
  Runtime,
  BuiltinResolver,
  BuiltinLoader,
  FileResolver,
  ScriptLoader,
} from "../dist/index.js";
