import { expect, test } from "vite-plus/test";
import {
  Context,
  realmEval,
  realmGlobalKeys,
  realmRegisterGlobalProperty,
  realmRunJobs,
} from "./main.ts";

test("Context default evaluation & console.log", () => {
  const ctx = new Context();
  const res = ctx.eval("console.log(3 + 3.33333)");
  expect(res).toBeNull();

  const numRes = ctx.eval("1 + 3");
  expect(numRes).toBe(4);
});

test("ContextBuilder configuration", () => {
  const builder = Context.builder();
  builder.canBlock(false);
  const ctx = builder.build();

  expect(ctx.canBlock()).toBe(false);
});

test("Context strict mode and state", () => {
  const ctx = new Context();
  ctx.strict(true);
  expect(() => {
    ctx.eval("x = 10;");
  }).toThrow();
});

test("Context global property and callable registration", () => {
  const ctx = new Context();
  ctx.registerGlobalProperty("myNum", 42, 7);
  expect(ctx.eval("myNum")).toBe(42);

  ctx.registerGlobalCallable("add", 2, ((a: any, b: any) => {
    return (a as number) + (b as number);
  }) as any);
  expect(ctx.eval("add(10, 20)")).toBe(30);

  ctx.registerGlobalBuiltinCallable("multiply", 2, ((a: any, b: any) => {
    return (a as number) * (b as number);
  }) as any);
  expect(ctx.eval("multiply(3, 4)")).toBe(12);

  const keys = ctx.globalKeys();
  expect(keys).toContain("console");
  expect(keys).toContain("myNum");
});

test("Context job executor & enqueueJob", () => {
  const ctx = new Context();
  let executed = false;
  ctx.enqueueJob(() => {
    executed = true;
  });
  expect(executed).toBe(false);
  ctx.runJobs();
  expect(executed).toBe(true);
});

test("Context limits, optimizer options, and stack trace", () => {
  const ctx = new Context();

  const limits = ctx.runtimeLimits();
  expect(limits).toBeDefined();
  ctx.setRuntimeLimits({ stackSize: 100000 });

  const opts = ctx.optimizerOptions();
  expect(opts).toBeDefined();
  ctx.setOptimizerOptions({ constantFolding: true });

  const trace = ctx.stackTrace();
  expect(Array.isArray(trace)).toBe(true);
});

test("Context host-defined data store", () => {
  const ctx = new Context();
  expect(ctx.hasData("key1")).toBe(false);

  ctx.insertData("key1", { value: 123 });
  expect(ctx.hasData("key1")).toBe(true);
  expect(ctx.getData("key1")).toEqual({ value: 123 });

  const removed = ctx.removeData("key1");
  expect(removed).toEqual({ value: 123 });
  expect(ctx.hasData("key1")).toBe(false);
});

test("Context clearKeptObjects", () => {
  const ctx = new Context();
  expect(() => ctx.clearKeptObjects()).not.toThrow();
});

test("Realm creation, switching, and operations", () => {
  const ctx = new Context();
  const initialRealm = ctx.realm();
  expect(initialRealm.id).toBeGreaterThan(0);
  expect(initialRealm.ctxId).toBeGreaterThan(0);

  const newRealm = ctx.createRealm();
  expect(newRealm.id).toBeGreaterThan(0);
  expect(newRealm.id).not.toBe(initialRealm.id);

  realmRegisterGlobalProperty(newRealm, "realmVar", "hello from realm", true, true, true);
  const evalRes = realmEval(newRealm, "realmVar");
  expect(evalRes).toBe("hello from realm");

  const realmKeys = realmGlobalKeys(newRealm);
  expect(realmKeys).toContain("realmVar");

  realmRunJobs(newRealm);

  const prevRealm = ctx.enterRealm(newRealm);
  expect(prevRealm.id).toBe(initialRealm.id);

  ctx.enterRealm(prevRealm);
});

test("Context dispose", () => {
  const ctx = new Context();
  ctx.dispose();
  expect(() => ctx.eval("1 + 1")).toThrow();
});
