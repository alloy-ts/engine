export function main(): string {
  return "Hello, world!";
}

export {
  Engine,
  Engine as Runtime,
  Global,
  Parser,
  add,
  Context,
  WeakRuntime,
  Loader,
} from "../dist/index.js";
