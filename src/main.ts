export function main(): string {
  return "Hello, world!";
}

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
} from "../dist/index.js";

export const PrimitiveType = {
  U32: "U32",
  I32: "I32",
  U64: "U64",
  I64: "I64",
  F64: "F64",
  F32: "F32",
  BOOL: "BOOL",
  STRING: "STRING",
} as const;

export type PrimitiveType = (typeof PrimitiveType)[keyof typeof PrimitiveType];
