/**
 * Tests for Atom NAPI bindings.
 */
import { describe, expect, test } from "vite-plus/test";
import { Atom, Context } from "./main.ts";

describe("napi: Atom", () => {
  test("creates Atom from string, numbers, and boolean in a Context", () => {
    const context = new Context(undefined);
    expect(context).toBeDefined();

    const strAtom = new Atom(context, "myKey");
    expect(strAtom.toString()).toBe("myKey");

    const u32Atom = Atom.fromU32(context, 42);
    expect(u32Atom.toString()).toBe("42");

    const i32Atom = Atom.fromI32(context, -10);
    expect(i32Atom.toString()).toBe("-10");

    const boolAtom = Atom.fromBool(context, true);
    expect(boolAtom.toString()).toBe("true");
  });
});
