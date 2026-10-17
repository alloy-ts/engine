import {
  Atom as NativeAtom,
  Compiler,
  Context,
  Engine,
  Global,
  Loader,
  Parser,
  Runtime,
  WeakRuntime,
  add
} from "../dist/index.js";

export function main(): string {
  return "Hello, world!";
}

const inJs = {
  Undefined: "undefined",
  Null: "null",
  False: "false",
  True: "true",
  Length: "length",
  Prototype: "prototype",
  Constructor: "constructor",
  Message: "message",
  Name: "name",
  Stack: "stack",
  Symbol: "Symbol",
  Object: "Object",
  Array: "Array",
  Error: "Error",
  Function: "Function",
  Promise: "Promise"
};

const inRs = {
  U32: "u32",
  I32: "i32",
  U64: "u64",
  I64: "i64",
  F64: "f64",
  F32: "f32",
  BOOL: "bool",
  STRING: "string"
};

(NativeAtom as unknown as { inJs: typeof inJs; inRs: typeof inRs }).inJs = inJs;
(NativeAtom as unknown as { inJs: typeof inJs; inRs: typeof inRs }).inRs = inRs;

export { NativeAtom as Atom, Compiler, Context, Engine, Global, Loader, Parser, Runtime, WeakRuntime, add };
