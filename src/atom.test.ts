import { describe, expect, test } from "vite-plus/test";
import { Atom } from "./main.ts";

describe("napi: Atom", () => {
  test("constructs Atom from constructor", () => {
    const atom = new Atom("myAtom");
    expect(atom).toBeDefined();
    expect(atom.toString()).toBe("myAtom");
  });

  test("creates Atom via static factory methods", () => {
    const atomStr = Atom.fromStr("foo");
    expect(atomStr.toString()).toBe("foo");

    const atomU32 = Atom.fromU32(123);
    expect(atomU32.toString()).toBe("123");

    const atomI32 = Atom.fromI32(-456);
    expect(atomI32.toString()).toBe("-456");

    const atomBool = Atom.fromBool(true);
    expect(atomBool.toString()).toBe("true");

    const atomF64 = Atom.fromF64(3.14);
    expect(atomF64.toString()).toBe("3.14");
  });

  test("converts Atom to JS value via toValue", () => {
    const atom = Atom.fromStr("bar");
    expect(atom.toValue()).toBe("bar");
  });

  test("checks equality between atoms", () => {
    const atom1 = Atom.fromStr("test");
    const atom2 = Atom.fromStr("test");
    const atom3 = Atom.fromStr("other");

    expect(atom1.equals(atom2)).toBe(true);
    expect(atom1.equals(atom3)).toBe(false);
  });
});
