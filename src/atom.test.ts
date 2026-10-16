import { describe, expect, test } from "vite-plus/test";
import { Atom } from "./main.ts";

describe("napi: Atom", () => {
  test("creates Atom from string", () => {
    const atom = Atom.fromStr("hello");
    expect(atom.toString()).toBe("hello");
  });

  test("creates Atom from numbers and booleans", () => {
    expect(Atom.fromU32(42).toString()).toBe("42");
    expect(Atom.fromI32(-10).toString()).toBe("-10");
    expect(Atom.fromBool(true).toString()).toBe("true");
    expect(Atom.fromF64(3.14).toString()).toBe("3.14");
  });
});
