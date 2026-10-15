import { expect, test } from "vite-plus/test";
import {
  Context,
  realmEval,
  realmGlobalKeys,
  realmGlobalObject,
  realmGlobalThis,
  realmRegisterProperty,
  realmRunJobs,
} from "./main.ts";

// Attribute bit flags — mirror `src/context.rs::attribute_from_u32`:
//   1 = writable, 2 = enumerable, 4 = configurable
const ATTR_ALL = 1 | 2 | 4;

// NOTE: these tests exercise the napi bindings produced by `build:napi`.
// They require the compiled native addon (`dist/index.js`); run `npm run
// build:napi` first. The TS `Env` parameter on the Rust side is injected by
// napi-derive and does not appear here; `JsFunction` becomes `Function`.

test("Context default + eval returns the completion value directly", () => {
  const ctx = new Context();
  // `eval` returns the script's completion value as JSON — no `console` needed.
  expect(ctx.eval("3 + 4.5")).toBe(7.5);
  expect(ctx.eval("2 ** 10")).toBe(1024);
  expect(ctx.eval('"hello" + " " + "world"')).toBe("hello world");
  expect(ctx.eval("1 + 1 === 2")).toBe(true);
  expect(ctx.eval("[1, 2, 3].length")).toBe(3);
});

test("Context.eval returns objects/arrays as JSON", () => {
  const ctx = new Context();
  expect(ctx.eval("({ a: 1, b: [2, 3] })")).toEqual({ a: 1, b: [2, 3] });
  expect(ctx.eval("Math.max(1, 9, 4)")).toBe(9);
  expect(ctx.eval("null")).toBe(null);
});

test("Context.strict toggles strict mode", () => {
  const ctx = new Context();
  ctx.strict(true);
  expect(
    ctx.eval("(() => { 'use strict'; return (function () { return this; })(); })()"),
  ).toBe(undefined);
  ctx.strict(false);
});

test("Context.registerGlobalProperty + globalObject/globalKeys", () => {
  const ctx = new Context();
  ctx.registerGlobalProperty("myVar", 42, ATTR_ALL);
  expect(ctx.eval("myVar")).toBe(42);
  const keys = ctx.globalKeys();
  expect(Array.isArray(keys)).toBe(true);
  expect(keys).toContain("myVar");
  expect(ctx.globalObject()).toBeDefined();
});

test("Context.registerGlobalCallable (callable + constructable)", () => {
  const ctx = new Context();
  // args/return are JSON-marshalled across the boundary.
  ctx.registerGlobalCallable("add", 2, (a: number, b: number) => a + b);
  expect(ctx.eval("add(2, 3)")).toBe(5);
  expect(ctx.eval("typeof new add(1, 2)")).toBe("object");
});

test("Context.registerGlobalBuiltinCallable (non-constructable)", () => {
  const ctx = new Context();
  ctx.registerGlobalBuiltinCallable("sub", 2, (a: number, b: number) => a - b);
  expect(ctx.eval("sub(10, 4)")).toBe(6);
  expect(() => ctx.eval("new sub(1, 2)")).toThrow();
});

test("Context.enqueueJob + runJobs", () => {
  const ctx = new Context();
  let ran = 0;
  ctx.enqueueJob(() => {
    ran += 1;
  });
  ctx.enqueueJob(() => {
    ran += 10;
  });
  expect(ran).toBe(0);
  ctx.runJobs();
  expect(ran).toBe(11);
});

test("Context.runtimeLimits / setRuntimeLimits", () => {
  const ctx = new Context();
  const rl = ctx.runtimeLimits();
  expect(typeof rl.stackSize).toBe("number");
  ctx.setRuntimeLimits({ recursionLimit: 1000 });
  expect(ctx.runtimeLimits().recursionLimit).toBe(1000);
});

test("Context.optimizerOptions / setOptimizerOptions", () => {
  const ctx = new Context();
  expect(ctx.optimizerOptions().constantFolding).toBe(true);
  ctx.setOptimizerOptions({ constantFolding: false });
  expect(ctx.optimizerOptions().constantFolding).toBe(false);
});

test("Context.canBlock", () => {
  const ctx = new Context();
  expect(ctx.canBlock()).toBe(false);
});

test("Context.stackTrace returns an array", () => {
  const ctx = new Context();
  expect(Array.isArray(ctx.stackTrace())).toBe(true);
});

test("Context host-defined data store (keyed)", () => {
  const ctx = new Context();
  ctx.insertData("k", 123);
  expect(ctx.hasData("k")).toBe(true);
  expect(ctx.getData("k")).toBe(123);
  expect(ctx.removeData("k")).toBe(123);
  expect(ctx.hasData("k")).toBe(false);
  expect(ctx.getData("k")).toBe(null);
});

test("Context.clearKeptObjects does not throw", () => {
  const ctx = new Context();
  expect(() => ctx.clearKeptObjects()).not.toThrow();
});

test("ContextBuilder builds a Context with options", () => {
  const ctx = Context.builder().canBlock(false).build();
  expect(ctx).toBeInstanceOf(Context);
  expect(ctx.canBlock()).toBe(false);
});

// ---- Realms ----
// `Realm` is a `#[napi(object)]` (a plain `{ id, ctxId }` handle), not a JS
// class, so we assert on its fields rather than using `instanceof`.

test("Realm: createRealm / realm / enterRealm", () => {
  const ctx = new Context();
  const defaultRealm = ctx.realm();
  expect(typeof defaultRealm.id).toBe("number");
  expect(defaultRealm.id).toBeGreaterThan(0);
  expect(typeof defaultRealm.ctxId).toBe("number");

  const r = ctx.createRealm();
  expect(typeof r.id).toBe("number");
  expect(r.id).toBeGreaterThan(0);

  const prev = ctx.enterRealm(r);
  expect(typeof prev.id).toBe("number");
  // active realm is now `r`
  expect(ctx.realm().id).toBe(r.id);
  // restore
  ctx.enterRealm(prev);
});

test("Realm free functions operate on a realm handle", () => {
  const ctx = new Context();
  const r = ctx.createRealm();
  expect(realmEval(r, "6 * 7")).toBe(42);

  realmRegisterProperty(r, "answer", 42, true, true, true);
  expect(realmEval(r, "answer")).toBe(42);

  const keys = realmGlobalKeys(r);
  expect(Array.isArray(keys)).toBe(true);
  expect(realmGlobalObject(r)).toBeDefined();
  expect(realmGlobalThis(r)).toBeDefined();
  realmRunJobs(r);
});

test("Realm.registerProperty: errors on non-configurable redefinition (boa semantics)", () => {
  const ctx = new Context();
  const r = ctx.createRealm();
  realmRegisterProperty(r, "x", 1, true, true, true); // configurable
  realmRegisterProperty(r, "x", 2, true, true, false); // now non-configurable
  // redefining a non-configurable property throws, mirroring
  // `define_property_or_throw`.
  expect(() => realmRegisterProperty(r, "x", 3, true, true, false)).toThrow();
});
