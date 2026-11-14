import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { Context, Engine } from "../dist/index.js";

const engine = new Engine();

void test("eval returns host primitives for the completion value", () => {
  const context = engine.createContext();
  assert.equal(context.eval(""), undefined);
  assert.equal(context.eval("1 + 2 * 3"), 7);
  assert.equal(context.eval("'a' + 1"), "a1");
  assert.equal(context.eval("1 < 2"), true);
  assert.equal(context.eval("null"), null);
  assert.equal(context.eval("undefined"), undefined);
  assert.equal(context.eval("1; 2\n'three'"), "three");
  assert.equal(context.eval("0.1 + 0.2"), 0.30000000000000004);
  assert.ok(Object.is(context.eval("-0"), -0));
  assert.ok(Object.is(context.eval("0 / 0"), NaN));
  assert.equal(context.eval("1 / 0"), Infinity);
  assert.equal(context.eval("-1 / 0"), -Infinity);
  assert.equal(context.eval("'' + 1e21"), "1e+21");
  assert.equal(context.eval("null == undefined"), true);
  assert.equal(context.eval("NaN === NaN"), false);
  assert.equal(context.eval("'😀' + '\\u00e9'"), "😀é");
  context.set("nul", "a\0b");
  assert.equal(context.get("nul"), "a\0b");
  assert.equal(context.eval("nul + nul"), "a\0ba\0b");
});

void test("throw completions become host errors of the matching type", () => {
  const context = engine.createContext();
  assert.throws(() => context.eval("1 +"), { name: "SyntaxError", message: "Unexpected end of input" });
  assert.throws(() => context.eval("'abc"), { name: "SyntaxError", message: "Unterminated string literal" });
  assert.throws(() => context.eval("(1"), SyntaxError);
  assert.throws(() => context.eval("missing"), {
    name: "ReferenceError",
    message: "missing is not defined",
  });
  assert.throws(() => context.eval("missing"), ReferenceError);
  // The context stays usable after a throw.
  assert.equal(context.eval("2"), 2);
});

void test("errors use the host constructors captured at addon load", () => {
  const original = globalThis.SyntaxError;
  const context = engine.createContext();
  try {
    globalThis.SyntaxError = class Fake extends Error {} as SyntaxErrorConstructor;
    assert.throws(() => context.eval(")"), original);
  } finally {
    globalThis.SyntaxError = original;
  }
});

void test("globals are context-local and round-trip primitives exactly", () => {
  const context = engine.createContext();
  const values: [string, unknown][] = [
    ["u", undefined],
    ["n", null],
    ["t", true],
    ["f", false],
    ["num", 42.5],
    ["negZero", -0],
    ["nan", NaN],
    ["inf", Infinity],
    ["negInf", -Infinity],
    ["max", Number.MAX_VALUE],
    ["str", "héllo 😀"],
    ["empty", ""],
  ];
  for (const [name, value] of values) {
    context.set(name, value as never);
    assert.ok(Object.is(context.get(name), value), name);
    assert.ok(Object.is(context.eval(name), value), name);
  }
  context.set("x", 20);
  context.set("x", 21);
  assert.equal(context.eval("x * 2"), 42);
  assert.equal(context.eval("str + '!'"), "héllo 😀!");
  assert.equal(context.get("Infinity"), Infinity);
  assert.throws(() => context.get("missing"), { name: "ReferenceError", message: "missing is not defined" });
});

void test("set rejects read-only and unreachable binding names", () => {
  const context = engine.createContext();
  for (const name of ["undefined", "NaN", "Infinity"]) {
    assert.throws(() => context.set(name, 1), {
      name: "TypeError",
      message: `Cannot assign to read only global '${name}'`,
    });
  }
  assert.equal(context.eval("undefined"), undefined);
  for (const name of ["", "1x", "a b", "true", "if"]) {
    assert.throws(() => context.set(name, 1), TypeError, name);
  }
});

void test("unsupported host values are rejected with TypeError", () => {
  const context = engine.createContext();
  const cases: [unknown, string][] = [
    [{}, "object"],
    [[1], "object"],
    [new Number(1), "object"],
    [() => 1, "function"],
    [Symbol("s"), "symbol"],
    [1n, "bigint"],
  ];
  for (const [value, kind] of cases) {
    assert.throws(() => context.set("v", value as never), {
      name: "TypeError",
      message: `Cannot transfer a host ${kind}; expected undefined | null | boolean | number | string`,
    });
  }
  assert.throws(() => context.set("v", "\ud800"), {
    name: "TypeError",
    message: "strings containing lone surrogates are not supported",
  });
  assert.throws(() => context.get("v"), ReferenceError);
});

void test("contexts are isolated from each other", () => {
  const a = engine.createContext();
  const b = new Engine().createContext();
  const c = engine.createContext();
  a.set("shared", 1);
  b.set("shared", "b");
  assert.equal(a.eval("shared"), 1);
  assert.equal(b.eval("shared"), "b");
  assert.throws(() => c.eval("shared"), ReferenceError);
  a.dispose();
  assert.equal(b.eval("shared"), "b");
});

void test("disposal makes every operation throw and is idempotent", () => {
  const context = engine.createContext();
  context.set("x", 1);
  assert.equal(context.disposed, false);
  context.dispose();
  context.dispose();
  assert.equal(context.disposed, true);
  for (const use of [() => context.eval("1"), () => context.get("x"), () => context.set("x", 2)]) {
    assert.throws(use, { name: "Error", message: "Context is disposed" });
  }
  assert.equal(engine.createContext().eval("1"), 1);
});

void test("Context is only created through an Engine", () => {
  assert.throws(() => new (Context as unknown as new () => unknown)());
});

void test("contexts never read or write the host globalThis", () => {
  const before = Reflect.ownKeys(globalThis).map(String).sort();
  const context = engine.createContext();
  context.set("hostLeak", 1);
  context.eval("hostLeak + 1");
  assert.equal("hostLeak" in globalThis, false);
  for (const name of ["globalThis", "process", "Number", "Engine", "Value"]) {
    assert.throws(() => context.eval(name), ReferenceError, name);
  }
  (globalThis as Record<string, unknown>).hostOnly = 5;
  try {
    assert.throws(() => context.eval("hostOnly"), ReferenceError);
  } finally {
    delete (globalThis as Record<string, unknown>).hostOnly;
  }
  assert.deepEqual(Reflect.ownKeys(globalThis).map(String).sort(), before);
});

void test("loading the addon and evaluating adds only the existing value globals", () => {
  const addon = fileURLToPath(new URL("../dist/index.js", import.meta.url));
  const output = execFileSync(
    process.execPath,
    [
      "--input-type=module",
      "-e",
      `const before = new Set(Reflect.ownKeys(globalThis).map(String));
       const { Engine } = await import(process.env.ADDON);
       const afterLoad = new Set(Reflect.ownKeys(globalThis).map(String));
       const context = new Engine().createContext();
       context.set("x", 1); context.eval("x + 1"); context.dispose();
       const afterEval = Reflect.ownKeys(globalThis).map(String);
       console.log(JSON.stringify({
         added: [...afterLoad].filter((k) => !before.has(k)).sort(),
         changedByEval: afterEval.length !== afterLoad.size || afterEval.some((k) => !afterLoad.has(k)),
       }));`,
    ],
    { env: { ...process.env, ADDON: addon }, encoding: "utf8" },
  );
  const result = JSON.parse(output) as { added: string[]; changedByEval: boolean };
  assert.equal(result.added.includes("Engine"), false);
  assert.equal(result.added.includes("Context"), false);
  assert.equal(result.changedByEval, false);
});
