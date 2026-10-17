import { describe, expect, test } from "vite-plus/test";
import { Atom } from "./main.ts";

describe("napi: Atom", () => {
  test("Atom.fromU32(123) works with JS primitives numbers", () => {
    const atom = Atom.fromU32(123);
    expect(atom).toBeDefined();
    expect(atom.toString()).toBe("123");
    expect(atom.to_string()).toBe("123");
    expect(atom.toValue()).toBe("123");
    expect(atom.to_value()).toBe("123");
  });

  test("Atom.fromI32(123).toString() works to invoke our to_string Rust method", () => {
    const atom = Atom.fromI32(123);
    expect(atom).toBeDefined();
    expect(atom.toString()).toBe("123");
    expect(atom.to_string()).toBe("123");
  });

  test("Atom.fromBool(true) works", () => {
    const atom = Atom.fromBool(true);
    expect(atom).toBeDefined();
    expect(atom.toString()).toBe("true");
  });

  test("Atom.fromF64(3.14) works", () => {
    const atom = Atom.fromF64(3.14);
    expect(atom).toBeDefined();
    expect(atom.toString()).toBe("3.14");
  });

  test("Atom.fromStr('foo') works", () => {
    const atom = Atom.fromStr("foo");
    expect(atom).toBeDefined();
    expect(atom.toString()).toBe("foo");
    expect(atom.toJsString()).toBe("foo");
    expect(atom.to_js_string()).toBe("foo");
  });

  test("Atom.fromValue('42') works", () => {
    const atom = Atom.fromValue("42");
    expect(atom).toBeDefined();
    expect(atom.toString()).toBe("42");
  });

  test("Atom.fromPredefined('undefined') works", () => {
    const atom = Atom.fromPredefined("undefined");
    expect(atom).toBeDefined();
    expect(atom.toString()).toBe("undefined");
  });

  test("Atom.inJs exports predefined atoms", () => {
    expect(Atom.inJs).toBeDefined();
    expect(Atom.inJs.Undefined).toBe("undefined");
    expect(Atom.inJs.Null).toBe("null");
    expect(Atom.inJs.Object).toBe("Object");
  });

  test("Atom.inRs exports native primitive types", () => {
    expect(Atom.inRs).toBeDefined();
    expect(Atom.inRs.U32).toBe("u32");
    expect(Atom.inRs.I32).toBe("i32");
    expect(Atom.inRs.U64).toBe("u64");
    expect(Atom.inRs.I64).toBe("i64");
    expect(Atom.inRs.F64).toBe("f64");
    expect(Atom.inRs.F32).toBe("f32");
    expect(Atom.inRs.BOOL).toBe("bool");
    expect(Atom.inRs.STRING).toBe("string");
  });
});
