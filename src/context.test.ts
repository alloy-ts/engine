import { expect, test } from "vite-plus/test";
import {
  Context,
  Engine,
  realmEval,
  realmGlobalKeys,
  realmRegisterProperty,
  realmRunJobs,
} from "./main.ts";

// Attribute bitmask used by `registerGlobalProperty` /
// `registerGlobalBuiltinCallable` (matches boa's `Attribute` bits):
//   1 = writable, 2 = enumerable, 4 = configurable   (7 = all)

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

test("new Context() default builds a boa-backed context", () => {
  const ctx = new Context();
  expect(ctx).toBeDefined();
  ctx.dispose();
});

test("Context.builder() produces a working context", () => {
  const b = Context.builder();
  b.canBlock(false);
  const ctx = b.build();
  expect(ctx.eval("7 * 6")).toBe(42);
  ctx.dispose();
});

// ---------------------------------------------------------------------------
// eval
// ---------------------------------------------------------------------------

test("eval returns the completion value (numbers)", () => {
  const ctx = new Context();
  expect(ctx.eval("1 + 1")).toBe(2);
  ctx.dispose();
});

test("console.log works inside eval (and returns the completion value)", () => {
  // `console` is registered as a global on every default context, so
  // `console.log(...)` prints to stdout and the statement returns 6.
  const ctx = new Context();
  const out = ctx.eval("console.log(3 + 3.33333); 6");
  expect(out).toBe(6);
  ctx.dispose();
});

test("eval handles strings, booleans, arrays, objects, null", () => {
  const ctx = new Context();
  expect(ctx.eval('"hello"')).toBe("hello");
  expect(ctx.eval("true")).toBe(true);
  expect(ctx.eval("null")).toBe(null);
  expect(ctx.eval("undefined")).toBe(null);
  expect(ctx.eval("[1, 2, 3]")).toEqual([1, 2, 3]);
  expect(ctx.eval("({ a: 1, b: 'x' })")).toEqual({ a: 1, b: "x" });
  ctx.dispose();
});

test("strict mode enforces restrictions", () => {
  const ctx = new Context();
  ctx.strict(true);
  expect(() => ctx.eval("'use strict'; x = 1")).toThrow();
  expect(ctx.eval("'use strict'; var y = 2; y")).toBe(2);
  ctx.dispose();
});

// ---------------------------------------------------------------------------
// Globals & registration
// ---------------------------------------------------------------------------

test("registerGlobalProperty + globalKeys", () => {
  const ctx = new Context();
  ctx.registerGlobalProperty("myVar", 42, 7);
  expect(ctx.eval("myVar")).toBe(42);
  expect(ctx.globalKeys()).toContain("myVar");
  expect(ctx.globalKeys()).toContain("console");
  ctx.dispose();
});

test("registerGlobalCallable bridges to a JS function", () => {
  const ctx = new Context();
  ctx.registerGlobalCallable("add", 2, (a: any, b: any) => a + b);
  expect(ctx.eval("typeof add")).toBe("function");
  expect(ctx.eval("add(2, 3)")).toBe(5);
  ctx.dispose();
});

test("registerGlobalBuiltinCallable is not a constructor", () => {
  const ctx = new Context();
  ctx.registerGlobalBuiltinCallable("sub", 2, (a: any, b: any) => a - b);
  expect(ctx.eval("sub(5, 2)")).toBe(3);
  expect(() => ctx.eval("new sub(1, 2)")).toThrow();
  ctx.dispose();
});

// ---------------------------------------------------------------------------
// Accessors / limits / options
// ---------------------------------------------------------------------------

test("runtime limits read/write", () => {
  const ctx = new Context();
  const before = ctx.runtimeLimits();
  expect(before).toHaveProperty("stackSize");
  ctx.setRuntimeLimits({ stackSize: 1_000_000 });
  expect(ctx.runtimeLimits().stackSize).toBe(1_000_000);
  ctx.dispose();
});

test("optimizer options read/write", () => {
  const ctx = new Context();
  const opts = ctx.optimizerOptions();
  expect(opts).toHaveProperty("constantFolding");
  ctx.setOptimizerOptions({ constantFolding: true });
  expect(ctx.optimizerOptions().constantFolding).toBe(true);
  ctx.dispose();
});

test("canBlock is false by default", () => {
  const ctx = new Context();
  expect(ctx.canBlock()).toBe(false);
  ctx.dispose();
});

test("stackTrace returns a frame array", () => {
  const ctx = new Context();
  expect(Array.isArray(ctx.stackTrace())).toBe(true);
  ctx.dispose();
});

test("globalObject is a JSON view of the global", () => {
  const ctx = new Context();
  const g = ctx.globalObject();
  expect(typeof g).toBe("object");
  ctx.dispose();
});

test("clearKeptObjects does not throw", () => {
  const ctx = new Context();
  expect(() => ctx.clearKeptObjects()).not.toThrow();
  ctx.dispose();
});

// ---------------------------------------------------------------------------
// Host-defined keyed data store
// ---------------------------------------------------------------------------

test("host-defined data store", () => {
  const ctx = new Context();
  ctx.insertData("k", 99);
  expect(ctx.hasData("k")).toBe(true);
  expect(ctx.getData("k")).toBe(99);
  expect(ctx.removeData("k")).toBe(99);
  expect(ctx.hasData("k")).toBe(false);
  ctx.dispose();
});

// ---------------------------------------------------------------------------
// Jobs
// ---------------------------------------------------------------------------

test("runJobs drains queued promises and enqueued callbacks", () => {
  const ctx = new Context();
  ctx.eval("Promise.resolve(1).then(v => { globalThis.__p = v; }); 0");
  ctx.runJobs();
  expect(ctx.eval("globalThis.__p")).toBe(1);

  let ran = false;
  ctx.enqueueJob(() => {
    ran = true;
  });
  ctx.runJobs();
  expect(ran).toBe(true);
  ctx.dispose();
});

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

test("each Context is isolated", () => {
  const a = new Context();
  a.registerGlobalProperty("x", 1, 7);
  const b = new Context();
  expect(() => b.eval("x")).toThrow();
  a.dispose();
  b.dispose();
});

test("disposed context throws on use", () => {
  const ctx = new Context();
  ctx.dispose();
  expect(() => ctx.eval("1")).toThrow();
});

// ---------------------------------------------------------------------------
// Realms (returned as opaque `{ id, ctxId }` handles)
// ---------------------------------------------------------------------------

test("createRealm/enterRealm/realm return Realm handles", () => {
  const ctx = new Context();
  const realm = ctx.createRealm();
  expect(realm).toHaveProperty("id");
  expect(realm).toHaveProperty("ctxId");
  const prev = ctx.enterRealm(realm);
  expect(prev).toHaveProperty("id");
  const current = ctx.realm();
  expect(current).toHaveProperty("id");
  ctx.dispose();
});

test("realmEval evaluates inside a realm", () => {
  const ctx = new Context();
  const realm = ctx.createRealm();
  expect(realmEval(realm, "1 + 1")).toBe(2);
  ctx.dispose();
});

test("realmRegisterProperty + realmGlobalKeys", () => {
  const ctx = new Context();
  const realm = ctx.createRealm();
  realmRegisterProperty(realm, "r", 7, true, true, true);
  expect(realmEval(realm, "r")).toBe(7);
  expect(realmGlobalKeys(realm)).toContain("r");
  ctx.dispose();
});

test("realmRunJobs drains promises in a realm", () => {
  const ctx = new Context();
  const realm = ctx.createRealm();
  realmEval(realm, "Promise.resolve(9).then(v => { globalThis.__rp = v; }); 0");
  realmRunJobs(realm);
  expect(realmEval(realm, "globalThis.__rp")).toBe(9);
  ctx.dispose();
});

// ---------------------------------------------------------------------------
// Engine API & Backend Selection
// ---------------------------------------------------------------------------

test("Engine default constructor and backend selection", () => {
  const defaultEngine = new Engine();
  expect(defaultEngine.backendName()).toBe("boa");
  expect(defaultEngine.eval("10 + 20")).toBe(30);

  const boaEngine = Engine.boa();
  expect(boaEngine.backendName()).toBe("boa");

  const qjsEngine = Engine.quickjs();
  expect(qjsEngine.backendName()).toBe("quickjs");

  expect(defaultEngine.backend).toEqual({
    boa: "boa",
    quickJs: "quickjs",
  });
});
