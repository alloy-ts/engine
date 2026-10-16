export function main(): string {
  return "Hello, world!";
}

export {
  Compiler,
  Context,
  Context as JSContext,
  Engine,
  FileResolver,
  Global,
  Parser,
  ScriptLoader,
  add,
} from "../dist/index.js";
