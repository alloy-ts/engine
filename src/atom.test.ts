import { describe, expect, test } from "vite-plus/test";
import { Atom } from "./main.ts";

describe("napi: Atom", () => {
  test("new Atom() constructs an Atom and converts to string/value", () => {
    const atom = new Atom("foo");
    expect(atom).toBeDefined();
    expect(atom.to_string()).toBe("foo");
    expect(atom.to_value()).toBe("foo");
  });

  test("Atom.ofType() creates predefined atom", () => {
    const atom = Atom.ofType("prototype");
    expect(atom).toBeDefined();
    expect(atom.to_string()).toBe("prototype");
    expect(atom.to_value()).toBe("prototype");
  });
});
