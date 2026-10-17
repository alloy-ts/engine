import { describe, expect, test } from "vite-plus/test";
import { Atom, Context } from "./main.ts";

describe("napi: Atom", () => {
  test("Atom.fromU32 / Atom.from_u32 works with JS primitive numbers", () => {
    const ctx = new Context();
    const atomCamel = Atom.fromU32(ctx, 123);
    expect(atomCamel).toBeDefined();
    expect(atomCamel.to_string()).toBe("123");
    expect(atomCamel.toString()).toBe("123");

    const atomSnake = Atom.from_u32(ctx, 456);
    expect(atomSnake).toBeDefined();
    expect(atomSnake.to_string()).toBe("456");
  });

  test("Atom.fromI32 / Atom.fromBool / Atom.fromF64 / Atom.fromStr works", () => {
    const ctx = new Context();
    const atomI32 = Atom.fromI32(ctx, -789);
    expect(atomI32.toString()).toBe("-789");

    const atomBool = Atom.fromBool(ctx, true);
    expect(atomBool.toString()).toBe("true");

    const atomF64 = Atom.fromF64(ctx, 3.14);
    expect(atomF64.toString()).toBe("3.14");

    const atomStr = Atom.fromStr(ctx, "hello_world");
    expect(atomStr.toString()).toBe("hello_world");
  });

  test("Atom.fromPredefined works", () => {
    const ctx = new Context();
    const predef = Atom.fromPredefined(ctx, "prototype");
    expect(predef.toString()).toBe("prototype");
  });
});
