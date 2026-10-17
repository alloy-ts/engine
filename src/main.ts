export function main(): string {
  return "Hello, world!";
}

import { Atom } from "../dist/index.js";

Object.defineProperty(Atom, "inJs", {
  get() {
    return {
      Undefined: "undefined",
      Null: "null",
      True: "true",
      False: "false",
      Length: "length",
      Name: "name",
      Object: "Object",
      Array: "Array",
      String: "String",
      Number: "Number",
      Boolean: "Boolean",
      Symbol: "Symbol",
      Function: "Function",
      Promise: "Promise",
      Value: "value",
      ToString: "toString",
      ValueOf: "valueOf",
    };
  },
  configurable: true,
  enumerable: true,
});

Object.defineProperty(Atom, "inRs", {
  get() {
    return {
      U32: "u32",
      I32: "i32",
      U64: "u64",
      I64: "i64",
      F64: "f64",
      F32: "f32",
      BOOL: "bool",
      STRING: "string",
    };
  },
  configurable: true,
  enumerable: true,
});

export {
  Engine,
  Global,
  Parser,
  add,
  Context,
  WeakEngine,
  Loader,
  Compiler,
  Atom,
  U32,
  I32,
  U64,
  I64,
  F64,
  F32,
  BOOL,
  STRING,
} from "../dist/index.js";
