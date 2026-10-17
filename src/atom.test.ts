/**
 * Tests for the embedded QuickJS `Atom` NAPI bindings (`src/atom.rs`).
 */
import { describe, expect, test } from "vite-plus/test";
import { Atom, type PrimitiveType } from "./main.ts";

describe("napi: Atom", () => {
  test("Atom.fromU32(123) works with JS primitives numbers", () => {
    const atom = Atom.fromU32(123);
    expect(atom).toBeDefined();
    expect(atom.toString()).toBe("123");
    expect(atom.to_js_string()).toBe("123");
    expect(atom.toValue()).toBe("123");
  });

  test("Atom.fromI32(-456) works", () => {
    const atom = Atom.fromI32(-456);
    expect(atom).toBeDefined();
    expect(atom.toString()).toBe("-456");
    expect(atom.toValue()).toBe("-456");
  });

  test("Atom.fromBool(true) and Atom.fromBool(false) work", () => {
    const atomTrue = Atom.fromBool(true);
    expect(atomTrue.toString()).toBe("true");

    const atomFalse = Atom.fromBool(false);
    expect(atomFalse.toString()).toBe("false");
  });

  test("Atom.fromF64(3.14) works", () => {
    const atom = Atom.fromF64(3.14);
    expect(atom.toString()).toBe("3.14");
    expect(atom.toValue()).toBe("3.14");
  });

  test("Atom.fromStr('foo') works", () => {
    const atom = Atom.fromStr("foo");
    expect(atom.toString()).toBe("foo");
    expect(atom.to_js_string()).toBe("foo");
    expect(atom.toValue()).toBe("foo");
  });

  test("Atom.fromPredefined('null') works", () => {
    const atom = Atom.fromPredefined("null");
    expect(atom.toString()).toBe("null");
  });

  test("Atom.fromValue('42') works", () => {
    const atom = Atom.fromValue("42");
    expect(atom.toString()).toBe("42");
  });

  test("new Atom('bar') constructor works", () => {
    const atom = new Atom("bar");
    expect(atom.toString()).toBe("bar");
  });

  test("PrimitiveType enum type exports expected values", () => {
    const u32Type: PrimitiveType = 0 as PrimitiveType;
    expect(u32Type).toBe(0);
  });
});
