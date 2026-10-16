/**
 * Tests for the `Atom` NAPI bindings (`src/atom.rs`).
 */
import { describe, expect, test } from "vite-plus/test";
import { Atom } from "./main.ts";

describe("napi: Atom", () => {
  test("Atom constructor and value getter", () => {
    const atom = new Atom("foo");
    expect(atom.value).toBe("foo");
    expect(atom.toString()).toBe("foo");
    expect(atom.toValue()).toBe("foo");
  });

  test("Atom.fromStr", () => {
    const atom = Atom.fromStr("bar");
    expect(atom.value).toBe("bar");
    expect(atom.toString()).toBe("bar");
  });

  test("Atom.fromU32", () => {
    const atom = Atom.fromU32(123);
    expect(atom.value).toBe("123");
  });

  test("Atom.fromI32", () => {
    const atom = Atom.fromI32(-456);
    expect(atom.value).toBe("-456");
  });

  test("Atom.fromBool", () => {
    const atom = Atom.fromBool(true);
    expect(atom.value).toBe("true");
  });

  test("Atom.fromF64", () => {
    const atom = Atom.fromF64(3.14);
    expect(atom.value).toBe("3.14");
  });
});
