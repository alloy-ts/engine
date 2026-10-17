import { describe, expect, test } from "vite-plus/test";
import { Atom } from "./main.ts";

describe("napi: Atom", () => {
  test("Atom.fromU32(123) works with JS primitive numbers", () => {
    const atom = Atom.fromU32(123);
    expect(atom).toBeDefined();
    expect(atom.toString()).toBe("123");
    expect(atom.toValue()).toBe("123");
  });

  test("Atom.fromI32(123).toString() works to invoke our to_string Rust method", () => {
    const atom = Atom.fromI32(123);
    expect(atom.toString()).toBe("123");
    expect(atom.toJsString()).toBe("123");
  });

  test("Atom.prototype.inJs / Atom.inJs contains predefined JS atoms such as Undefined", () => {
    const atom = new Atom("test");
    const inJs = atom.inJs;
    expect(inJs).toBeDefined();
    expect(inJs.Undefined).toBe("undefined");
    expect(inJs.Null).toBe("null");
    expect(inJs.True).toBe("true");
    expect(inJs.False).toBe("false");
  });

  test("Atom.prototype.inRs / Atom.inRs contains native primitive types", () => {
    const atom = new Atom("test");
    const inRs = atom.inRs;
    expect(inRs).toBeDefined();
    expect(inRs.U32).toBe("u32");
    expect(inRs.I32).toBe("i32");
    expect(inRs.U64).toBe("u64");
    expect(inRs.I64).toBe("i64");
    expect(inRs.F64).toBe("f64");
    expect(inRs.F32).toBe("f32");
    expect(inRs.BOOL).toBe("bool");
    expect(inRs.STRING).toBe("string");
  });

  test("Atom factory methods: fromBool, fromF64, fromStr, fromValue, fromPredefined", () => {
    expect(Atom.fromBool(true).toString()).toBe("true");
    expect(Atom.fromF64(3.14).toString()).toBe("3.14");
    expect(Atom.fromStr("test").toString()).toBe("test");
    expect(Atom.fromValue("'quickjs'").toString()).toBe("quickjs");
    expect(Atom.fromPredefined("null").toString()).toBe("null");
  });
});
