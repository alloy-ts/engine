import { describe, expect, test } from "vite-plus/test";
import { Atom, U32, I32, U64, I64, F64, F32, BOOL, STRING } from "./main.ts";

describe("napi: Atom", () => {
  test("Atom.fromU32(123) works with JS primitives numbers", () => {
    const atom = Atom.fromU32(123);
    expect(atom).toBeDefined();
    expect(atom.toString()).toBe("123");
    expect(atom.toValue()).toBe("123");
  });

  test("Atom.fromI32(123).toString() works to invoke our to_string Rust method", () => {
    const atom = Atom.fromI32(123);
    expect(atom.toString()).toBe("123");
    expect(atom.to_string()).toBe("123");
  });

  test("Atom factory methods for other primitives", () => {
    const atomBool = Atom.fromBool(true);
    expect(atomBool.toString()).toBe("true");

    const atomF64 = Atom.fromF64(3.14);
    expect(atomF64.toString()).toBe("3.14");

    const atomStr = Atom.fromStr("hello");
    expect(atomStr.toString()).toBe("hello");

    const atomVal = Atom.fromValue("789");
    expect(atomVal.toString()).toBe("789");

    const atomPredef = Atom.fromPredefined("null");
    expect(atomPredef.toString()).toBe("null");
  });

  test("Atom.prototype methods", () => {
    const atom = Atom.fromStr("test_atom");
    expect(atom.toString()).toBe("test_atom");
    expect(atom.toJsString()).toBe("test_atom");
    expect(atom.toValue()).toBe("test_atom");
    expect(atom.to_string()).toBe("test_atom");
    expect(atom.to_js_string()).toBe("test_atom");
    expect(atom.to_value()).toBe("test_atom");
  });

  test("Atom.inJs static property", () => {
    expect(Atom.inJs).toBeDefined();
    expect(Atom.inJs.Undefined).toBe("undefined");
    expect(Atom.inJs.Null).toBe("null");
    expect(Atom.inJs.True).toBe("true");
    expect(Atom.inJs.False).toBe("false");
    expect(Atom.inJs.Length).toBe("length");
  });

  test("Atom.inRs static property", () => {
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

  test("primitive type identifier constants", () => {
    expect(U32).toBe("u32");
    expect(I32).toBe("i32");
    expect(U64).toBe("u64");
    expect(I64).toBe("i64");
    expect(F64).toBe("f64");
    expect(F32).toBe("f32");
    expect(BOOL).toBe("bool");
    expect(STRING).toBe("string");
  });
});
