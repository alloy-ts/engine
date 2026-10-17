export function main(): string {
  return "Hello, world!";
}

import {
  Engine,
  Global,
  Parser,
  add,
  Context,
  ContextBuilder,
  WeakEngine,
  Loader,
  Compiler,
  Atom,
  type PredefinedAtom,
  type NativePrimitive,
} from "../dist/index.js";

if (Atom) {
  try {
    if (typeof (Atom as any).inJs === "function") {
      const jsAtoms = (Atom as any).inJs();
      Object.defineProperty(Atom, "inJs", {
        value: jsAtoms,
        writable: true,
        configurable: true,
        enumerable: true,
      });
    }
  } catch {}
  try {
    if (typeof (Atom as any).inRs === "function") {
      const rsPrimitives = (Atom as any).inRs();
      Object.defineProperty(Atom, "inRs", {
        value: rsPrimitives,
        writable: true,
        configurable: true,
        enumerable: true,
      });
    }
  } catch {}
}

export {
  Engine,
  Global,
  Parser,
  add,
  Context,
  ContextBuilder,
  WeakEngine,
  Loader,
  Compiler,
  Atom,
  type PredefinedAtom,
  type NativePrimitive,
};
