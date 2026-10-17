export function main(): string {
  return "Hello, world!";
}

import { Atom as NativeAtom } from "../dist/index.js";

Object.defineProperty(NativeAtom, "inJs", {
  get() {
    return NativeAtom.prototype.inJs;
  },
  configurable: true,
  enumerable: true,
});

Object.defineProperty(NativeAtom, "inRs", {
  get() {
    return {
      U32: (ctx: any, val: number) => NativeAtom.fromU32(ctx, val),
      I32: (ctx: any, val: number) => NativeAtom.fromI32(ctx, val),
      U64: (ctx: any, val: bigint) => NativeAtom.fromU64(ctx, val),
      I64: (ctx: any, val: bigint) => NativeAtom.fromI64(ctx, val),
      F64: (ctx: any, val: number) => NativeAtom.fromF64(ctx, val),
      F32: (ctx: any, val: number) => NativeAtom.fromF32(ctx, val),
      BOOL: (ctx: any, val: boolean) => NativeAtom.fromBool(ctx, val),
      STRING: (ctx: any, val: string) => NativeAtom.fromStr(ctx, val),
    };
  },
  configurable: true,
  enumerable: true,
});

export const Atom = NativeAtom;

export {
  add,
  Compiler,
  Context,
  Engine,
  Global,
  Loader,
  Parser,
  PredefinedAtom,
  WeakEngine,
} from "../dist/index.js";
