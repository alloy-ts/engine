import { expect, test } from "vite-plus/test";
import {
  Context,
  ContextBuilder,
  realmEval,
  realmGlobalKeys,
  realmRegisterGlobalProperty,
  realmRunJobs,
} from "./main.ts";

test("Context default evaluation including console.log", () => {
  const ctx = new Context();
  const res = ctx.eval("console.log(3 + 3.33333)");
  expect(res).toBeNull();

  const val = ctx.eval("3 + 3.33333");
  expect(val).toBeCloseTo(6.33333);
});

test("ContextBuilder creates context", () => {
  const builder = new ContextBuilder();
  builder.canBlock(false);
  const ctx = builder.build();
  expect(ctx.canBlock()).toBe(false);
  expect(ctx.eval("10 * 20")).toBe(200);
});

test("Context global property and global keys", () => {
  const ctx = new Context();
  ctx.registerGlobalProperty("myNum", 42, 7); // ALL = 7
  expect(ctx.eval("myNum")).toBe(42);

  const keys = ctx.globalKeys();
  expect(keys).toContain("console");
  expect(keys).toContain("myNum");
});

test("Context register global callable", () => {
  const ctx = new Context();
  let called = false;
  ctx.registerGlobalCallable("addTen", 1, (x: unknown) => {
    called = true;
    return (x as number) + 10;
  });

  const res = ctx.eval("addTen(15)");
  expect(called).toBe(true);
  expect(res).toBe(25);
});

test("Context register global builtin callable", () => {
  const ctx = new Context();
  ctx.registerGlobalBuiltinCallable("greet", 1, (name: unknown) => {
    return `Hello, ${name}`;
  });

  expect(ctx.eval("greet('World')")).toBe("Hello, World");
});

test("Context strict mode and clear kept objects", () => {
  const ctx = new Context();
  ctx.strict(true);
  expect(() => ctx.eval("x = 100")).toThrow();

  ctx.clearKeptObjects();
});

test("Context limits and optimizer options", () => {
  const ctx = new Context();
  const limits = ctx.runtimeLimits();
  expect(limits).toHaveProperty("stackSize");

  ctx.setRuntimeLimits({ loopIterationLimit: 5000 });
  expect(ctx.runtimeLimits().loopIterationLimit).toBe(5000);

  const opts = ctx.optimizerOptions();
  expect(opts).toHaveProperty("constantFolding");

  ctx.setOptimizerOptions({ constantFolding: true, deadCodeElimination: false });
  expect(ctx.optimizerOptions().constantFolding).toBe(true);
});

test("Context host defined data store", () => {
  const ctx = new Context();
  ctx.insertData("user", { name: "Alice", id: 1 });
  expect(ctx.hasData("user")).toBe(true);
  expect(ctx.getData("user")).toEqual({ name: "Alice", id: 1 });

  const removed = ctx.removeData("user");
  expect(removed).toEqual({ name: "Alice", id: 1 });
  expect(ctx.hasData("user")).toBe(false);
});

test("Context job queue and runJobs", () => {
  const ctx = new Context();
  let run = false;
  ctx.enqueueJob(() => {
    run = true;
  });
  expect(run).toBe(false);
  ctx.runJobs();
  expect(run).toBe(true);
});

test("Context stack trace", () => {
  const ctx = new Context();
  const trace = ctx.stackTrace();
  expect(Array.isArray(trace)).toBe(true);
});

test("Realm operations via Context", () => {
  const ctx = new Context();
  const initialRealm = ctx.realm();
  expect(initialRealm).toHaveProperty("id");
  expect(initialRealm).toHaveProperty("ctxId");

  const newRealm = ctx.createRealm();
  expect(newRealm.id).not.toBe(initialRealm.id);

  realmRegisterGlobalProperty(newRealm, "realmVal", 99, true, true, true);
  const evalRes = realmEval(newRealm, "realmVal * 2");
  expect(evalRes).toBe(198);

  const keys = realmGlobalKeys(newRealm);
  expect(keys).toContain("realmVal");

  realmRunJobs(newRealm);

  const prev = ctx.enterRealm(newRealm);
  expect(prev.id).toBe(initialRealm.id);
  expect(ctx.realm().id).toBe(newRealm.id);

  ctx.enterRealm(prev);
  expect(ctx.realm().id).toBe(initialRealm.id);
});

test("Context dispose", () => {
  const ctx = new Context();
  expect(ctx.eval("1 + 1")).toBe(2);
  ctx.dispose();
  expect(() => ctx.eval("1 + 1")).toThrow();
});
