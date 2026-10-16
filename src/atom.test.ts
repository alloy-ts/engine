import { describe, expect, test } from "vite-plus/test";
import { Atom, PredefinedAtom } from "./main.ts";

describe("napi: Atom", () => {
  test("new Atom() and Atom.from_str create an Atom instance", () => {
    const atom1 = new Atom("foo");
    expect(atom1).toBeDefined();
    expect(atom1.to_string()).toBe("foo");
    expect(atom1.toString()).toBe("foo");

    const atom2 = Atom.from_str("bar");
    expect(atom2.to_string()).toBe("bar");
  });

  test("Atom.of_type creates an Atom from PredefinedAtom", () => {
    const atomNull = Atom.of_type(PredefinedAtom.Null);
    expect(atomNull.to_string()).toBe("null");

    const atomProto = Atom.of_type(PredefinedAtom.Prototype);
    expect(atomProto.to_string()).toBe("prototype");
  });
});
