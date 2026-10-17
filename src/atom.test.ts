import { describe, expect, test } from "vite-plus/test";
import { Atom, Context, PredefinedAtom } from "./main.ts";

describe("napi: Atom", () => {
  test("Atom.fromU32(123) works with JS primitive numbers", () => {
    const ctx = new Context();
    const atom = Atom.fromU32(ctx, 123);
    expect(atom).toBeDefined();
    expect(atom.toValue()).toBe(123);
    expect(atom.toString()).toBe("123");
    expect(atom.toJsString()).toBe("123");
  });

  test("Atom.fromI32(123).toString() works to invoke our to_string Rust method", () => {
    const ctx = new Context();
    const atom = Atom.fromI32(ctx, 123);
    expect(atom.toString()).toBe("123");
    expect(atom.toValue()).toBe(123);
  });

  test("Atom.fromBool and Atom.fromF64 work", () => {
    const ctx = new Context();
    const boolAtom = Atom.fromBool(ctx, true);
    expect(boolAtom.toString()).toBe("true");

    const floatAtom = Atom.fromF64(ctx, 3.14);
    expect(floatAtom.toString()).toBe("3.14");
  });

  test("Atom.fromStr and Atom.fromValue work", () => {
    const ctx = new Context();
    const strAtom = Atom.fromStr(ctx, "hello");
    expect(strAtom.toString()).toBe("hello");
    expect(strAtom.toValue()).toBe("hello");

    const valAtom = Atom.fromValue(ctx, "world");
    expect(valAtom.toString()).toBe("world");
    expect(valAtom.toValue()).toBe("world");
  });

  test("Atom.fromPredefined works", () => {
    const ctx = new Context();
    const preAtom = Atom.fromPredefined(ctx, PredefinedAtom.Undefined);
    expect(preAtom.toString()).toBe("undefined");
  });

  test("Atom.inJs exports predefined atoms", () => {
    expect(Atom.inJs).toBeDefined();
    expect(Atom.inJs.Undefined).toBe("Undefined");
    expect(Atom.inJs.Null).toBe("Null");
  });

  test("Atom.inRs exports native primitive constructors", () => {
    const ctx = new Context();
    expect(Atom.inRs).toBeDefined();
    expect(typeof Atom.inRs.U32).toBe("function");
    expect(typeof Atom.inRs.I32).toBe("function");
    expect(typeof Atom.inRs.U64).toBe("function");
    expect(typeof Atom.inRs.I64).toBe("function");
    expect(typeof Atom.inRs.F64).toBe("function");
    expect(typeof Atom.inRs.F32).toBe("function");
    expect(typeof Atom.inRs.BOOL).toBe("function");
    expect(typeof Atom.inRs.STRING).toBe("function");

    const atomU32 = Atom.inRs.U32(ctx, 123);
    expect(atomU32.toValue()).toBe(123);

    const atomI32 = Atom.inRs.I32(ctx, -123);
    expect(atomI32.toString()).toBe("-123");

    const atomBool = Atom.inRs.BOOL(ctx, true);
    expect(atomBool.toString()).toBe("true");

    const atomStr = Atom.inRs.STRING(ctx, "alloy");
    expect(atomStr.toString()).toBe("alloy");
  });
});
