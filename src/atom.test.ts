import { describe, expect, test } from "vite-plus/test";
import { Atom, Context } from "./main.ts";

describe("napi: Atom", () => {
  test("creates an atom from string and converts back to string", () => {
    const ctx = new Context();
    const atom = Atom.from_str(ctx, "hello_atom");
    expect(atom).toBeDefined();
    expect(atom.to_string()).toBe("hello_atom");
  });
});
