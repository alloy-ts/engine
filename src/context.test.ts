import { expect, test } from "vite-plus/test";
import { Context, realmEval, realmGlobalObject, realmRegisterProperty } from "./main.ts";

test("Context default construction and eval console.log", () => {
  const ctx = new Context(); // default
  const result = ctx.eval("console.log(3 + 3.33333)");
  expect(result).toBe(null); // console.log returns undefined, which marshals to null
});

test("Context.eval basic expressions and values", () => {
  const ctx = new Context();
  expect(ctx.eval("1 + 2")).toBe(3);
  expect(ctx.eval("'hello' + ' world'")).toBe("hello world");
  expect(ctx.eval("true && false")).toBe(false);
  expect(ctx.eval("({ a: 10, b: [1, 2, 3] })")).toEqual({ a: 10, b: [1, 2, 3] });
});

test("Context.registerGlobalProperty", () => {
  const ctx = new Context();
  // WRITABLE (1) | ENUMERABLE (2) | CONFIGURABLE (4) = 7
  ctx.registerGlobalProperty("myNum", 42, 7);
  expect(ctx.eval("myNum")).toBe(42);

  ctx.registerGlobalProperty("myStr", "hello", 7);
  expect(ctx.eval("myStr")).toBe("hello");
});

test("Context.registerGlobalCallable and registerGlobalBuiltinCallable", () => {
  const ctx = new Context();
  let calledWith: unknown[] = [];

  ctx.registerGlobalCallable("addNum", 2, (a: number, b: number) => {
    calledWith = [a, b];
    return a + b;
  });

  expect(ctx.eval("addNum(5, 7)")).toBe(12);
  expect(calledWith).toEqual([5, 7]);

  ctx.registerGlobalBuiltinCallable("builtinFunc", 1, (x: number) => x * 2);
  expect(ctx.eval("builtinFunc(21)")).toBe(42);
  expect(() => ctx.eval("new builtinFunc(21)")).toThrow();
});

test("Context.globalKeys and globalObject", () => {
  const ctx = new Context();
  ctx.registerGlobalProperty("fooKey", "barVal", 7);

  const keys = ctx.globalKeys();
  expect(keys).toContain("fooKey");
  expect(keys).toContain("console");

  const obj = ctx.globalObject() as Record<string, unknown>;
  expect(obj.fooKey).toBe("barVal");
});

test("Context.strict mode", () => {
  const ctx = new Context();
  ctx.strict(true);
  // In strict mode, assigning to an undeclared variable throws SyntaxError/ReferenceError
  expect(() => ctx.eval("undeclaredVar = 100")).toThrow();

  ctx.strict(false);
});

test("Context.enqueueJob and runJobs", () => {
  const ctx = new Context();
  let ran = false;

  ctx.enqueueJob(() => {
    ran = true;
  });

  expect(ran).toBe(false);
  ctx.runJobs();
  expect(ran).toBe(true);
});

test("Context.canBlock", () => {
  const ctx = new Context();
  expect(typeof ctx.canBlock()).toBe("boolean");
  expect(ctx.canBlock()).toBe(false);
});

test("Context.runtimeLimits and setRuntimeLimits", () => {
  const ctx = new Context();
  const limits = ctx.runtimeLimits();
  expect(limits).toHaveProperty("stackSize");
  expect(limits).toHaveProperty("loopIterationLimit");

  ctx.setRuntimeLimits({
    loopIterationLimit: 1000,
  });
  const updated = ctx.runtimeLimits();
  expect(updated.loopIterationLimit).toBe(1000);
});

test("Context.optimizerOptions and setOptimizerOptions", () => {
  const ctx = new Context();
  const opts = ctx.optimizerOptions();
  expect(typeof opts).toBe("object");

  ctx.setOptimizerOptions({
    constantFolding: true,
    deadCodeElimination: false,
  });
  const updated = ctx.optimizerOptions();
  expect(updated.constantFolding).toBe(true);
  expect(updated.deadCodeElimination).toBe(false);
});

test("Context.stackTrace", () => {
  const ctx = new Context();
  const trace = ctx.stackTrace();
  expect(Array.isArray(trace)).toBe(true);
});

test("Context realms API", () => {
  const ctx = new Context();
  const defaultRealm = ctx.realm();
  expect(defaultRealm).toHaveProperty("id");
  expect(defaultRealm).toHaveProperty("ctxId");

  const newRealm = ctx.createRealm();
  expect(newRealm.id).not.toBe(defaultRealm.id);

  realmRegisterProperty(newRealm, "realmVar", 99, true, true, true);
  expect(realmEval(newRealm, "realmVar")).toBe(99);

  const prev = ctx.enterRealm(newRealm);
  expect(prev.id).toBe(defaultRealm.id);
  expect(ctx.eval("realmVar")).toBe(99);

  ctx.enterRealm(prev);
  expect(ctx.eval("typeof realmVar")).toBe("undefined");
});

test("Context host-defined data store", () => {
  const ctx = new Context();
  expect(ctx.hasData("key1")).toBe(false);

  ctx.insertData("key1", { test: 123 });
  expect(ctx.hasData("key1")).toBe(true);
  expect(ctx.getData("key1")).toEqual({ test: 123 });

  const removed = ctx.removeData("key1");
  expect(removed).toEqual({ test: 123 });
  expect(ctx.hasData("key1")).toBe(false);
});

test("Context.clearKeptObjects", () => {
  const ctx = new Context();
  expect(() => ctx.clearKeptObjects()).not.toThrow();
});

test("ContextBuilder", () => {
  const builder = Context.builder();
  builder.canBlock(false);
  const ctx = builder.build();
  expect(ctx.eval("2 + 2")).toBe(4);
});

test("Context.dispose", () => {
  const ctx = new Context();
  expect(ctx.eval("1 + 1")).toBe(2);
  ctx.dispose();
  expect(() => ctx.eval("1 + 1")).toThrow("Context has been disposed");
});
