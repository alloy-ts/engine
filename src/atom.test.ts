import { describe, expect, test } from "vite-plus/test";
import { Atom } from "./main.ts";

describe("napi: Atom", () => {
  test("Atom.fromU32 and Atom.from_u32 work with JS primitive numbers", () => {
    const atomCamel = Atom.fromU32(123);
    expect(atomCamel.to_string()).toBe("123");
    expect(atomCamel.toString()).toBe("123");

    const atomSnake = Atom.from_u32(123);
    expect(atomSnake.to_string()).toBe("123");
  });

  test("Atom static factories support all primitive types", () => {
    expect(Atom.fromI32(-42).toString()).toBe("-42");
    expect(Atom.fromBool(true).toString()).toBe("true");
    expect(Atom.fromF64(3.14).toString()).toBe("3.14");
    expect(Atom.fromStr("hello").toString()).toBe("hello");
    expect(Atom.fromPredefined("null").toString()).toBe("null");
    expect(Atom.fromValue(100).toString()).toBe("100");
  });
});
