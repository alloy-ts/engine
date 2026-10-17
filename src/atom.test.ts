import { describe, expect, test } from "vite-plus/test";
import { Atom } from "./main.ts";

describe("napi: Atom unit tests", () => {
  test("Atom.fromU32(123) works with JS primitive numbers", () => {
    const atom = Atom.fromU32(123);
    expect(atom).toBeDefined();
    expect(atom.toString()).toBe("123");
  });

  test("Atom.fromI32(123).toString() works to invoke Rust to_string method", () => {
    const atom = Atom.fromI32(123);
    expect(atom.toString()).toBe("123");
    expect(atom.to_string()).toBe("123");
  });

  test("Atom.fromBool and Atom.fromF64 work", () => {
    const bAtom = Atom.fromBool(true);
    expect(bAtom.toString()).toBe("true");

    const fAtom = Atom.fromF64(3.14);
    expect(fAtom.toString()).toBe("3.14");
  });

  test("Atom.fromStr and Atom.fromPredefined work", () => {
    const strAtom = Atom.fromStr("foo");
    expect(strAtom.toString()).toBe("foo");
    expect(strAtom.toJsString()).toBe("foo");
    expect(strAtom.to_js_string()).toBe("foo");

    const predefAtom = Atom.fromPredefined("Undefined");
    expect(predefAtom.toString()).toBe("Undefined");
  });

  test("Atom.fromValue and toValue work", () => {
    const valAtom = Atom.fromValue("hello");
    expect(valAtom.toValue()).toEqual("hello");
  });

  test("Atom.inJs and Atom.inRs expose predefined constants", () => {
    const inJs = Atom.inJs as { Undefined: string; Null: string };
    expect(inJs.Undefined).toBe("undefined");
    expect(inJs.Null).toBe("null");

    const inRs = Atom.inRs as { U32: string; I32: string; BOOL: string; STRING: string };
    expect(inRs.U32).toBe("u32");
    expect(inRs.I32).toBe("i32");
    expect(inRs.BOOL).toBe("bool");
    expect(inRs.STRING).toBe("string");
  });
});
