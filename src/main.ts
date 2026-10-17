export function main(): string {
  return "Hello, world!";
}

import {
  Atom,
  Compiler,
  Context,
  Global,
  JsEngine as Engine,
  JsLoader as Loader,
  JsWeakEngine as WeakEngine,
  Parser,
} from "../dist/index.js";

if (typeof Atom !== "undefined" && Atom.prototype) {
  Object.defineProperty(Atom, "inJs", {
    get() {
      return (Atom.prototype as any).inJs;
    },
    configurable: true,
    enumerable: true,
  });
  Object.defineProperty(Atom, "in_js", {
    get() {
      return (Atom.prototype as any).inJs;
    },
    configurable: true,
    enumerable: true,
  });
  Object.defineProperty(Atom, "inRs", {
    get() {
      return (Atom.prototype as any).inRs;
    },
    configurable: true,
    enumerable: true,
  });
  Object.defineProperty(Atom, "in_rs", {
    get() {
      return (Atom.prototype as any).inRs;
    },
    configurable: true,
    enumerable: true,
  });
}

export { Atom, Compiler, Context, Engine, Global, Loader, Parser, WeakEngine };
export type { PredefinedAtom } from "../dist/index.js";
