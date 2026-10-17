import { describe, expect, test } from "vite-plus/test";
import { Atom, PrimitiveType } from "./main.ts";

describe("napi: Atom", () => {
  test("static constructors from_value and from_str", () => {
    const atomStr = Atom.from_str("hello");
    expect(atomStr.to_string()).toBe("hello");
    expect(atomStr.to_js_string()).toBe("hello");
    expect(atomStr.to_value()).toBe("hello");

    const atomVal = Atom.from_value("42");
    expect(atomVal.to_string()).toBe("42");
  });

  test("static numeric and boolean constructors", () => {
    const u32Atom = Atom.from_u32(100);
    expect(u32Atom.to_string()).toBe("100");

    const i32Atom = Atom.from_i32(-50);
    expect(i32Atom.to_string()).toBe("-50");

    const boolAtom = Atom.from_bool(true);
    expect(boolAtom.to_string()).toBe("true");

    const f64Atom = Atom.from_f64(3.14);
    expect(f64Atom.to_string()).toBe("3.14");

    const f32Atom = Atom.from_f32(2.5);
    expect(f32Atom.to_string()).toBe("2.5");
  });

  test("static predefined constructor", () => {
    const predef = Atom.from_predefined("null");
    expect(predef.to_string()).toBe("null");
  });

  test("native primitive type static factory constructors", () => {
    expect(Atom.U32(42).to_string()).toBe("42");
    expect(Atom.I32(-10).to_string()).toBe("-10");
    expect(Atom.U64(1000).to_string()).toBe("1000");
    expect(Atom.I64(-500).to_string()).toBe("-500");
    expect(Atom.F64(1.23).to_string()).toBe("1.23");
    expect(Atom.F32(4.56).to_string()).toBe("4.56");
    expect(Atom.BOOL(false).to_string()).toBe("false");
    expect(Atom.STRING("world").to_string()).toBe("world");
  });

  test("PrimitiveType enum is exported", () => {
    expect(PrimitiveType.U32).toBeDefined();
    expect(PrimitiveType.I32).toBeDefined();
    expect(PrimitiveType.BOOL).toBeDefined();
    expect(PrimitiveType.STRING).toBeDefined();
  });
});
