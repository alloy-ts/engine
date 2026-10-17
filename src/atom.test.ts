import { describe, expect, test } from "vite-plus/test";
import { Atom } from "./main.ts";

describe("napi: Atom", () => {
  test("new Atom() creates an atom instance", () => {
    const atom = new Atom("hello");
    expect(atom).toBeDefined();
    expect(atom.to_string()).toBe("hello");
    expect(atom.to_js_string()).toBe("hello");
    expect(atom.to_value()).toBe("hello");
  });

  test("Atom default constructor with empty value", () => {
    const atom = new Atom();
    expect(atom.to_string()).toBe("");
  });

  test("Atom.from_u32 creates an atom from u32", () => {
    const atom = Atom.from_u32(123);
    expect(atom.to_string()).toBe("123");
  });

  test("Atom.from_i32 creates an atom from i32", () => {
    const atom = Atom.from_i32(-456);
    expect(atom.to_string()).toBe("-456");
  });

  test("Atom.from_bool creates an atom from bool", () => {
    const atomTrue = Atom.from_bool(true);
    const atomFalse = Atom.from_bool(false);
    expect(atomTrue.to_string()).toBe("true");
    expect(atomFalse.to_string()).toBe("false");
  });

  test("Atom.from_f64 creates an atom from f64", () => {
    const atom = Atom.from_f64(3.14159);
    expect(atom.to_string()).toBe("3.14159");
  });

  test("Atom.from_str creates an atom from string", () => {
    const atom = Atom.from_str("alloy");
    expect(atom.to_string()).toBe("alloy");
  });

  test("Atom.from_value creates an atom from JavaScript value representation", () => {
    const atom = Atom.from_value("'quickjs'");
    expect(atom.to_string()).toBe("quickjs");
  });

  test("Atom.from_predefined creates an atom from a predefined atom name", () => {
    const atom = Atom.from_predefined("null");
    expect(atom.to_string()).toBe("null");
  });
});
