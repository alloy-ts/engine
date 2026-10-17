import { describe, expect, test } from "vite-plus/test";
import { Atom, Context } from "./main.ts";

describe("napi: Atom", () => {
  test("Atom.fromU32(123) works with JS primitive numbers and toString() invokes to_string", () => {
    const atomU32 = Atom.fromU32(123);
    expect(atomU32).toBeInstanceOf(Atom);
    expect(atomU32.toString()).toBe("123");
    expect(atomU32.to_string()).toBe("123");

    const atomI32 = Atom.fromI32(-456);
    expect(atomI32).toBeInstanceOf(Atom);
    expect(atomI32.toString()).toBe("-456");
    expect(atomI32.to_string()).toBe("-456");
  });

  test("Atom factory constructors: from_u32, from_i32, from_bool, from_f64, from_str, from_value", () => {
    const ctx = new Context();

    const atomU32 = Atom.from_u32(ctx, 42);
    expect(atomU32).toBeInstanceOf(Atom);
    expect(atomU32.to_string()).toBe("42");
    expect(atomU32.toString()).toBe("42");
    expect(atomU32.to_js_string()).toBe("42");
    expect(atomU32.toJsString()).toBe("42");
    expect(atomU32.to_value()).toBe(42);
    expect(atomU32.toValue()).toBe(42);

    const atomI32 = Atom.from_i32(ctx, -100);
    expect(atomI32.to_string()).toBe("-100");
    expect(atomI32.toValue()).toBe(-100);

    const atomBool = Atom.from_bool(ctx, true);
    expect(atomBool.to_string()).toBe("true");
    expect(atomBool.toValue()).toBe(true);

    const atomF64 = Atom.from_f64(ctx, 3.14);
    expect(atomF64.to_string()).toBe("3.14");
    expect(atomF64.toValue()).toBe(3.14);

    const atomStr = Atom.from_str(ctx, "hello");
    expect(atomStr.to_string()).toBe("hello");
    expect(atomStr.toValue()).toBe("hello");

    const atomValStr = Atom.from_value(ctx, "world");
    expect(atomValStr.to_string()).toBe("world");

    const atomValNum = Atom.from_value(ctx, 123);
    expect(atomValNum.to_string()).toBe("123");
  });

  test("Atom factory constructor: from_predefined", () => {
    const ctx = new Context();

    const atomPredefEnum = Atom.from_predefined(ctx, 47 /* PredefinedAtom.Length */);
    expect(atomPredefEnum).toBeInstanceOf(Atom);
    expect(atomPredefEnum.to_string()).toBe("length");

    const atomPredefStr = Atom.from_predefined(ctx, "toString");
    expect(atomPredefStr.to_string()).toBe("toString");
  });

  test("Atom.inJs.* and Atom.inRs.* static properties", () => {
    const inJs =
      typeof (Atom as any).inJs === "function" ? (Atom as any).inJs() : (Atom as any).inJs;
    const inRs =
      typeof (Atom as any).inRs === "function" ? (Atom as any).inRs() : (Atom as any).inRs;

    expect(inJs).toBeDefined();
    expect(inJs.Undefined).toBe(66);
    expect(inJs.Length).toBe(47);

    expect(inRs).toBeDefined();
    expect(inRs.U32).toBe(0);
    expect(inRs.STRING).toBe(7);
  });

  test("Atom primitive type constructors: U32, I32, U64, I64, F64, F32, BOOL, STRING", () => {
    const ctx = new Context();

    expect(Atom.U32(ctx, 10).to_string()).toBe("10");
    expect(Atom.u32(10).to_string()).toBe("10");

    expect(Atom.I32(ctx, -20).to_string()).toBe("-20");
    expect(Atom.i32(-20).to_string()).toBe("-20");

    expect(Atom.U64(1000).to_string()).toBe("1000");
    expect(Atom.u64(1000).to_string()).toBe("1000");

    expect(Atom.I64(-500).to_string()).toBe("-500");
    expect(Atom.i64(-500).to_string()).toBe("-500");

    expect(Atom.F64(2.718).to_string()).toBe("2.718");
    expect(Atom.f64(2.718).to_string()).toBe("2.718");

    expect(Atom.F32(1.5).to_string()).toBe("1.5");
    expect(Atom.f32(1.5).to_string()).toBe("1.5");

    expect(Atom.BOOL(false).to_string()).toBe("false");
    expect(Atom.bool(false).to_string()).toBe("false");

    expect(Atom.STRING("test").to_string()).toBe("test");
    expect(Atom.string("test").to_string()).toBe("test");
  });
});
