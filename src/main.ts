export function main(): string {
  return "Hello, world!";
}

import { Atom as NativeAtom, Engine, Global, Parser, add, Context, WeakEngine, Loader, Compiler } from "../dist/index.js";

const inJsDict: Record<string, string> = {
  Undefined: "undefined",
  Null: "null",
  False: "false",
  True: "true",
  Object: "Object",
  Array: "Array",
  String: "String",
  Number: "Number",
  Boolean: "Boolean",
  Symbol: "Symbol",
  Function: "Function",
  Error: "Error",
  Promise: "Promise",
  Length: "length",
  Prototype: "prototype",
  Constructor: "constructor",
  Value: "value",
  Name: "name",
  Message: "message",
  Stack: "stack",
  ToString: "toString",
  ValueOf: "valueOf",
  Eval: "eval",
  GlobalThis: "globalThis",
};

const inRsDict: Record<string, string> = {
  U32: "u32",
  I32: "i32",
  U64: "u64",
  I64: "i64",
  F64: "f64",
  F32: "f32",
  BOOL: "bool",
  STRING: "string",
};

Object.defineProperty(NativeAtom, "inJs", {
  get() {
    return inJsDict;
  },
  configurable: true,
  enumerable: true,
});

Object.defineProperty(NativeAtom, "inRs", {
  get() {
    return inRsDict;
  },
  configurable: true,
  enumerable: true,
});

export const Atom = NativeAtom;
export { Engine, Global, Parser, add, Context, WeakEngine, Loader, Compiler };
