export function main(): string {
  return "Hello, world!";
}

export {
  Atom,
  BOOL,
  BuiltinModuleLoader,
  BuiltinModuleResolver,
  Compiler,
  Context,
  Engine,
  F32,
  F64,
  FileModuleResolver,
  Global,
  I32,
  I64,
  Parser,
  STRING,
  ScriptModuleLoader,
  U32,
  U64,
  add,
} from "../dist/index.js";
