import { describe, expect, test } from "vite-plus/test";
import { Atom } from "./main.ts";

describe("napi: Atom", () => {
  test("new Atom() creates an atom instance", () => {
    const atom = new Atom("hello");
    expect(atom).toBeDefined();
    expect(atom.toString()).toBe("hello");
    expect(atom.toJsString()).toBe("hello");
    expect(atom.toValue()).toBe("hello");
  });

  test("Atom default constructor with empty value", () => {
    const atom = new Atom();
    expect(atom.toString()).toBe("");
  });

  test("Atom.fromU32 creates an atom from u32", () => {
    const atom = Atom.fromU32(123);
    expect(atom.toString()).toBe("123");
  });

  test("Atom.fromI32 creates an atom from i32", () => {
    const atom = Atom.fromI32(-456);
    expect(atom.toString()).toBe("-456");
  });

  test("Atom.fromBool creates an atom from bool", () => {
    const atomTrue = Atom.fromBool(true);
    const atomFalse = Atom.fromBool(false);
    expect(atomTrue.toString()).toBe("true");
    expect(atomFalse.toString()).toBe("false");
  });

  test("Atom.fromF64 creates an atom from f64", () => {
    const atom = Atom.fromF64(3.14159);
    expect(atom.toString()).toBe("3.14159");
  });

  test("Atom.fromStr creates an atom from string", () => {
    const atom = Atom.fromStr("alloy");
    expect(atom.toString()).toBe("alloy");
  });

  test("Atom.fromValue creates an atom from JavaScript value representation", () => {
    const atom = Atom.fromValue("'quickjs'");
    expect(atom.toString()).toBe("quickjs");
  });

  test("Atom.fromPredefined creates an atom from a predefined atom name", () => {
    const atom = Atom.fromPredefined("null");
    expect(atom.toString()).toBe("null");
  });
});
