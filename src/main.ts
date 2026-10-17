export function main(): string {
  return "Hello, world!";
}

import { Atom as NativeAtom, Engine, Global, Parser, add, Context, WeakEngine, Loader, Compiler } from "../dist/index.js";

const inJs = {
  Undefined: "undefined",
  Null: "null",
  False: "false",
  True: "true",
  Length: "length",
  ToString: "toString",
  ValueOf: "valueOf",
  Prototype: "prototype",
  Constructor: "constructor",
};

const inRs = {
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
  value: inJs,
  writable: false,
  enumerable: true,
  configurable: true,
});

Object.defineProperty(NativeAtom, "inRs", {
  value: inRs,
  writable: false,
  enumerable: true,
  configurable: true,
});

export { Engine, Global, Parser, add, Context, WeakEngine, Loader, Compiler, NativeAtom as Atom };
