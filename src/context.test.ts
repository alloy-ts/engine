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

// Attribute bitmask used by `registerGlobalProperty`
// (matches boa's `Attribute` bits):
//   1 = writable, 2 = enumerable, 4 = configurable   (7 = all)

// ---------------------------------------------------------------------------
// Note on `console.log`: boa's `console` is registered from Rust and its `log`
// writes straight to the process stdout fd via `println!`, so its output is
// visible in the test runner's stdout but is NOT interceptable from JS. We
// therefore assert the observable JS contract (it runs without throwing and
// evaluates to `undefined`) rather than capturing the printed bytes.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

test("new Context() default builds a boa-backed context", () => {
  const ctx = new Context();
  expect(ctx).toBeDefined();
  ctx.dispose();
});

test("Context.builder().build() produces a working context", () => {
  const ctx = Context.builder().build();
  expect(ctx.eval("7 * 6")).toBe(42);
  ctx.dispose();
});

test("ContextBuilder.canBlock(true) is reflected in the built context", () => {
  const builder = Context.builder();
  builder.canBlock(true);
  const ctx = builder.build();
  expect(ctx.canBlock()).toBe(true);
  ctx.dispose();
});

test("default context is not allowed to block", () => {
  const ctx = new Context();
  expect(ctx.canBlock()).toBe(false);
  ctx.dispose();
});

// ---------------------------------------------------------------------------
// eval — including the exact console.log example
// ---------------------------------------------------------------------------

test("eval returns the completion value (numbers, strings, booleans)", () => {
  const ctx = new Context();
  expect(ctx.eval("1 + 1")).toBe(2);
  expect(ctx.eval('"hello"')).toBe("hello");
  expect(ctx.eval("true")).toBe(true);
  expect(ctx.eval("false")).toBe(false);
  ctx.dispose();
});

test("eval preserves floating point precision", () => {
  const ctx = new Context();
  expect(ctx.eval("3 + 3.33333")).toBe(6.33333);
  expect(ctx.eval("0.1 + 0.2")).toBe(0.30000000000000004);
  ctx.dispose();
});

test("eval maps structured values through JSON", () => {
  const ctx = new Context();
  expect(ctx.eval("[1, 2, 3]")).toEqual([1, 2, 3]);
  expect(ctx.eval("({ a: 1, b: 'x', c: [true, null] })")).toEqual({
    a: 1,
    b: "x",
    c: [true, null],
  });
  expect(ctx.eval("null")).toBe(null);
  expect(ctx.eval("undefined")).toBe(null);
  ctx.dispose();
});

// The literal snippet from the task. It PRINTS `6.33333` to stdout (visible in
// the runner output) and returns the completion value of `console.log(...)`,
// which is `undefined` → marshalled to `null`.
test('ctx.eval("console.log(3 + 3.33333)") prints 6.33333 and returns null', () => {
  const ctx = new Context(); // default
  const ret = ctx.eval("console.log(3 + 3.33333)");
  expect(ret).toBe(null); // console.log returns undefined
  ctx.dispose();
});

test("console.log runs and the trailing expression is the completion value", () => {
  const ctx = new Context();
  // If `console.log` threw, eval would throw; the completion value is the
  // trailing expression `3 + 3.33333` → 6.33333.
  expect(ctx.eval("console.log(3 + 3.33333); 3 + 3.33333")).toBe(6.33333);
  ctx.dispose();
});

test("console.error / console.warn are available", () => {
  const ctx = new Context();
  // These write to stderr; just assert they exist and are callable.
  expect(ctx.eval("typeof console.error")).toBe("function");
  expect(ctx.eval("typeof console.warn")).toBe("function");
  expect(ctx.eval("typeof console.log")).toBe("function");
  ctx.dispose();
});

test("eval throws a JS exception on a runtime error", () => {
  const ctx = new Context();
  expect(() => ctx.eval("throw new TypeError('boom')")).toThrow(/boom/);
  expect(() => ctx.eval("notDefinedAtAll")).toThrow();
  ctx.dispose();
});

test("eval supports BigInt expressions (via typeof)", () => {
  const ctx = new Context();
  expect(ctx.eval("typeof (2n ** 70n)")).toBe("bigint");
  ctx.dispose();
});

// ---------------------------------------------------------------------------
// strict mode
// ---------------------------------------------------------------------------

test("strict mode enforces restrictions", () => {
  const ctx = new Context();
  ctx.strict(true);
  expect(() => ctx.eval("'use strict'; x = 1")).toThrow();
  expect(ctx.eval("'use strict'; var y = 2; y")).toBe(2);
  ctx.dispose();
});

test("strict can be turned back off", () => {
  const ctx = new Context();
  ctx.strict(true);
  ctx.strict(false);
  expect(ctx.eval("'use strict'; var z = 3; z")).toBe(3);
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
  ctx.registerGlobalCallable("add", 2, (a: number, b: number) => a + b);
  expect(ctx.eval("typeof add")).toBe("function");
  expect(ctx.eval("add(2, 3)")).toBe(5);
  ctx.dispose();
});

test("registerGlobalCallable passes many args and complex return values", () => {
  const ctx = new Context();
  ctx.registerGlobalCallable(
    "sum3",
    3,
    (a: number, b: number, c: number) => a + b + c,
  );
  expect(ctx.eval("sum3(1, 2, 3)")).toBe(6);
  ctx.registerGlobalCallable("makeObj", 0, () => ({ ok: true, n: 7 }));
  expect(ctx.eval("makeObj()")).toEqual({ ok: true, n: 7 });
  ctx.dispose();
});

test("registerGlobalBuiltinCallable is not a constructor", () => {
  const ctx = new Context();
  ctx.registerGlobalBuiltinCallable("sub", 2, (a: number, b: number) => a - b);
  expect(ctx.eval("sub(5, 2)")).toBe(3);
  expect(() => ctx.eval("new sub(1, 2)")).toThrow();
  ctx.dispose();
});

// ---------------------------------------------------------------------------
// Accessors
// ---------------------------------------------------------------------------

test("globalObject is a JSON view of the global", () => {
  const ctx = new Context();
  const g = ctx.globalObject();
  expect(typeof g).toBe("object");
  expect(g).not.toBe(null);
  ctx.dispose();
});

test("stackTrace returns a frame array", () => {
  const ctx = new Context();
  expect(Array.isArray(ctx.stackTrace())).toBe(true);
  ctx.dispose();
});

// ---------------------------------------------------------------------------
// Runtime limits
// ---------------------------------------------------------------------------

test("runtime limits read/write", () => {
  const ctx = new Context();
  const before = ctx.runtimeLimits();
  expect(before).toHaveProperty("stackSize");
  ctx.setRuntimeLimits({ stackSize: 1_000_000, recursionLimit: 400 });
  const after = ctx.runtimeLimits();
  expect(after.stackSize).toBe(1_000_000);
  expect(after.recursionLimit).toBe(400);
  ctx.dispose();
});

test("recursion limit actually bounds evaluation", () => {
  const ctx = new Context();
  ctx.setRuntimeLimits({ recursionLimit: 32 });
  expect(() => ctx.eval("function f(){ return f(); } f();")).toThrow();
  ctx.dispose();
});

// ---------------------------------------------------------------------------
// Optimizer options
// ---------------------------------------------------------------------------

test("optimizer options read/write", () => {
  const ctx = new Context();
  const opts = ctx.optimizerOptions();
  expect(opts).toHaveProperty("constantFolding");
  ctx.setOptimizerOptions({ constantFolding: true, statistics: true });
  const after = ctx.optimizerOptions();
  expect(after.constantFolding).toBe(true);
  expect(after.statistics).toBe(true);
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

test("host-defined data store round-trips values", () => {
  const ctx = new Context();
  ctx.insertData("k", 99);
  expect(ctx.hasData("k")).toBe(true);
  expect(ctx.getData("k")).toBe(99);
  expect(ctx.removeData("k")).toBe(99);
  expect(ctx.hasData("k")).toBe(false);
  ctx.dispose();
});

test("host-defined data store handles structured + missing keys", () => {
  const ctx = new Context();
  ctx.insertData("obj", { a: 1, b: [2, 3] });
  expect(ctx.getData("obj")).toEqual({ a: 1, b: [2, 3] });
  expect(ctx.getData("missing")).toBe(null);
  expect(ctx.removeData("missing")).toBe(null);
  expect(ctx.hasData("obj")).toBe(true);
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
// Lifecycle & isolation
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

test("realmGlobalObject / realmGlobalThis expose the realm global", () => {
  const ctx = new Context();
  const realm = ctx.createRealm();
  realmRegisterProperty(realm, "marker", 123, true, true, true);
  const obj = realmGlobalObject(realm);
  const glo = realmGlobalThis(realm);
  expect(typeof obj).toBe("object");
  expect(obj).not.toBe(null);
  expect(typeof glo).toBe("object");
  expect(glo).not.toBe(null);
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
