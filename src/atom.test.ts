import { describe, expect, test } from "vite-plus/test";
import { Atom, Context } from "./main.ts";

describe("napi: Atom", () => {
  test("creates an Atom from a string", () => {
    const ctx = new Context();
    const atom = Atom.from_string(ctx, "hello");
    expect(atom).toBeDefined();
    expect(atom.toString()).toBe("hello");
  });

  test("creates an Atom from u32 index", () => {
    const ctx = new Context();
    const atom = Atom.from_u32(ctx, 42);
    expect(atom).toBeDefined();
    expect(atom.toString()).toBe("42");
  });
});
