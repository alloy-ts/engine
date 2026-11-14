import assert from "node:assert/strict";
import { test } from "node:test";
import vm from "node:vm";
import { Engine } from "../dist/index.js";

type Primitive = undefined | null | boolean | number | string;

type Outcome =
  | { type: "normal"; value: Primitive }
  | { type: "throw-error"; name: string }
  | { type: "throw-value"; value: Primitive };

interface Fixture {
  name: string;
  sources: string[];
}

const fixtures: Fixture[] = [
  { name: "numeric precedence", sources: ["1 + 2 * 3", "(1 + 2) * 3"] },
  { name: "number edge cases", sources: ["0 / 0", "1 / 0", "-0", "-4 % 2"] },
  { name: "primitive coercion", sources: ["'1' + 2", "'6' * '7'", "null + 1"] },
  { name: "equality", sources: ["null == undefined", "NaN === NaN", "+0 === -0"] },
  { name: "persistent lexical binding", sources: ["let x = 1", "x += 2", "x"] },
  { name: "persistent var binding", sources: ["var x = 1", "x += 2", "x"] },
  { name: "temporal dead zone", sources: ["{ x; let x = 1; }"] },
  { name: "redeclaration early error", sources: ["let x; var x;"] },
  { name: "typeof unresolvable reference", sources: ["typeof missing"] },
  {
    name: "logical short circuit",
    sources: ["let x = 0; false && (x = 1); x", "true || missing"],
  },
  { name: "if completion", sources: ["if (true) 1; else 2", "if (false) 1"] },
  {
    name: "while break and continue",
    sources: [
      "let x = 0; let sum = 0; while (x < 5) { x += 1; if (x === 2) continue; sum += x; if (x === 4) break; } sum",
    ],
  },
  {
    name: "label targets",
    sources: [
      "let x = 0; outer: while (x < 3) { x += 1; while (true) { continue outer; } } x",
      "done: { 7; break done; 9; }",
    ],
  },
  {
    name: "catch binding and finally preservation",
    sources: [
      "try { throw 1; } catch (error) { error + 1; } finally { 9; }",
      "try { missing; } catch (error) { typeof error; }",
    ],
  },
  { name: "abrupt finally replacement", sources: ["try { throw 1; } finally { throw 2; }"] },
  { name: "primitive throw", sources: ["throw undefined", "throw null", "throw 42", "throw 'x'"] },
];

function isPrimitive(value: unknown): value is Primitive {
  return (
    value === undefined ||
    value === null ||
    typeof value === "boolean" ||
    typeof value === "number" ||
    typeof value === "string"
  );
}

function outcome(run: () => unknown): Outcome {
  try {
    const value = run();
    assert.ok(isPrimitive(value), `fixture produced unsupported normal value: ${typeof value}`);
    return { type: "normal", value };
  } catch (thrown) {
    if (isPrimitive(thrown)) return { type: "throw-value", value: thrown };
    if (
      typeof thrown === "object" &&
      thrown !== null &&
      "name" in thrown &&
      typeof thrown.name === "string"
    ) {
      return { type: "throw-error", name: thrown.name };
    }
    throw thrown;
  }
}

for (const fixture of fixtures) {
  void test(`ECMA differential: ${fixture.name}`, () => {
    const alloy = new Engine().createContext();
    const node = vm.createContext(Object.create(null) as object);

    for (const source of fixture.sources) {
      const expected = outcome(() => vm.runInContext(source, node));
      const actual = outcome(() => alloy.eval(source));
      assert.deepEqual(actual, expected, source);
    }
  });
}

void test("documented deviation: lone UTF-16 surrogates remain host-only", () => {
  const alloy = new Engine().createContext();
  const node = vm.createContext(Object.create(null) as object);
  const source = "'\\uD800'";

  assert.deepEqual(
    outcome(() => vm.runInContext(source, node)),
    {
      type: "normal",
      value: "\uD800",
    },
  );
  assert.deepEqual(
    outcome(() => alloy.eval(source)),
    {
      type: "throw-error",
      name: "SyntaxError",
    },
  );
});
