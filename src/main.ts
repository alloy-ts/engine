export function main(): string {
  return "Hello, world!";
}

export {
  Context,
  Context as JSContext,
  Engine,
  FileResolver,
  Global,
  Parser,
  Runtime,
  ScriptLoader,
  add,
} from "../dist/index.js";
