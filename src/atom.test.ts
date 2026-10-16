import { describe, expect, test } from "vite-plus/test";
import { Atom } from "./main.ts";

describe("napi: Atom", () => {
  test("constructs Atom from string, u32, i32, bool, and f64", () => {
    const a1 = Atom.fromStr("foo");
    expect(a1.toString()).toBe("foo");

    const a2 = Atom.fromU32(100);
    expect(a2.toString()).toBe("100");

    const a3 = Atom.fromI32(-42);
    expect(a3.toString()).toBe("-42");

    const a4 = Atom.fromBool(true);
    expect(a4.toString()).toBe("true");

    const a5 = Atom.fromF64(3.14);
    expect(a5.toString()).toBe("3.14");
  });

  test("converts Atom to JS string and JS value", () => {
    const atom = new Atom("bar");
    expect(atom.toJsString()).toBe("bar");
    expect(atom.toValue()).toBe("bar");
  });
});
