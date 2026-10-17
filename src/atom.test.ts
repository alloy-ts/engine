import { describe, expect, test } from "vite-plus/test";
import { Atom, Context } from "./main.ts";

describe("napi: Atom", () => {
  test("Atom.fromU32(123) works with JS primitive numbers", () => {
    const ctx = new Context();
    const atom = Atom.fromU32(ctx, 123);
    expect(atom).toBeDefined();
    expect(atom.to_string()).toBe("123");
  });

  test("Atom.fromI32(123).toString() works to invoke our to_string Rust method", () => {
    const ctx = new Context();
    const atom = Atom.fromI32(ctx, 123);
    expect(atom).toBeDefined();
    expect(atom.to_string()).toBe("123");
    expect(atom.to_js_string()).toBe("123");
    expect(atom.to_value()).toBe("123");
  });

  test("Atom.from_u32 and Atom.from_i32 work with snake_case names", () => {
    const ctx = new Context();
    const uAtom = Atom.from_u32(ctx, 456);
    expect(uAtom.to_string()).toBe("456");

    const iAtom = Atom.from_i32(ctx, -789);
    expect(iAtom.to_string()).toBe("-789");
  });

  test("Atom.fromBool / Atom.from_bool works", () => {
    const ctx = new Context();
    const bAtom1 = Atom.fromBool(ctx, true);
    expect(bAtom1.to_string()).toBe("true");

    const bAtom2 = Atom.from_bool(ctx, false);
    expect(bAtom2.to_string()).toBe("false");
  });

  test("Atom.fromF64 / Atom.from_f64 works", () => {
    const ctx = new Context();
    const fAtom1 = Atom.fromF64(ctx, 3.14);
    expect(fAtom1.to_string()).toBe("3.14");

    const fAtom2 = Atom.from_f64(ctx, 2.718);
    expect(fAtom2.to_string()).toBe("2.718");
  });

  test("Atom.fromStr / Atom.from_str works", () => {
    const ctx = new Context();
    const sAtom1 = Atom.fromStr(ctx, "hello");
    expect(sAtom1.to_string()).toBe("hello");

    const sAtom2 = Atom.from_str(ctx, "world");
    expect(sAtom2.to_string()).toBe("world");
  });

  test("Atom.fromValue / Atom.from_value works with primitives and JSON", () => {
    const ctx = new Context();
    const vAtom1 = Atom.fromValue(ctx, "atomValue");
    expect(vAtom1.to_string()).toBe("atomValue");

    const vAtom2 = Atom.from_value(ctx, "snakeAtomValue");
    expect(vAtom2.to_string()).toBe("snakeAtomValue");

    const vAtom3 = Atom.fromValue(ctx, 42);
    expect(vAtom3.to_string()).toBe("42");

    const vAtom4 = Atom.fromValue(ctx, true);
    expect(vAtom4.to_string()).toBe("true");

    const vAtom5 = Atom.fromValue(ctx, null);
    expect(vAtom5.to_string()).toBe("null");
  });

  test("Atom.fromPredefined / Atom.from_predefined works and handles invalid IDs safely", () => {
    const ctx = new Context();
    const pAtom1 = Atom.fromPredefined(ctx, "length");
    expect(pAtom1.to_string()).toBe("length");

    const pAtom2 = Atom.from_predefined(ctx, 1); // 1 is PredefinedAtom::Null as u32
    expect(pAtom2.to_string()).toBe("null");

    expect(() => Atom.from_predefined(ctx, 999999)).toThrow("Invalid predefined atom ID");
  });

  test("Atom.inJs and Atom.inRs namespaces exist", () => {
    const inJs = Atom.inJs;
    expect(inJs).toBeDefined();
    expect(inJs.Undefined).toBe("undefined");
    expect(inJs.Null).toBe("null");

    const inRs = Atom.inRs;
    expect(inRs).toBeDefined();
    expect(inRs.U32).toBe("u32");
    expect(inRs.I32).toBe("i32");
    expect(inRs.STRING).toBe("string");
  });
});
