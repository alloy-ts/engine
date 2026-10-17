import { expect, test } from "vite-plus/test";
import { Atom, Context } from "./main.ts";

test("Atom.fromU32(123) works with JS primitives numbers", () => {
  const ctx = new Context();
  const atom = Atom.fromU32(ctx, 123);
  expect(atom).toBeInstanceOf(Atom);
  expect(atom.toValue()).toBe(123);
});

test("Atom.fromI32(123).toString() works to invoke our to_string Rust method", () => {
  const ctx = new Context();
  const atom = Atom.fromI32(ctx, 123);
  expect(atom.toString()).toBe("123");
  expect(atom.to_string()).toBe("123");
});

test("Atom factory constructors and prototype methods", () => {
  const ctx = new Context();

  const aFromStr = Atom.fromStr(ctx, "hello");
  expect(aFromStr.toString()).toBe("hello");

  const aFromBool = Atom.fromBool(ctx, true);
  expect(aFromBool.toString()).toBe("true");

  const aFromPredef = Atom.fromPredefined(ctx, "Undefined" as any);
  expect(aFromPredef.toString()).toBe("undefined");

  expect((Atom as any).inJs.Undefined).toBeDefined();
  expect(typeof (Atom as any).inRs.U32).toBe("function");
});
