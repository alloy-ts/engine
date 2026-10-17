import { expect, test } from "vite-plus/test";
import {
  Context,
  ContextBuilder,
  realmEval,
  realmGlobalKeys,
  realmGlobalObject,
  realmGlobalThis,
  realmRegisterProperty,
  realmRunJobs,
} from "./main.ts";

test("Context default construction and basic eval with console.log", () => {
  const ctx = new Context();
  // Prompt requirement test case
  const res = ctx.eval("console.log(3 + 3.33333)");
  expect(res).toBe(null); // console.log returns undefined, JSON-marshalled as null
});

test("Context.eval evaluates expressions", () => {
  const ctx = new Context();
  expect(ctx.eval("1 + 2")).toBe(3);
  expect(ctx.eval("'hello ' + 'world'")).toBe("hello world");
  expect(ctx.eval("({ a: 1, b: 'test' })")).toEqual({ a: 1, b: "test" });
});

test("ContextBuilder constructs context with custom options", () => {
  const builder = Context.builder();
  builder.canBlock(false);
  const ctx = builder.build();
  expect(ctx.canBlock()).toBe(false);
});

test("Context strict mode", () => {
  const ctx = new Context();
  ctx.strict(true);
  expect(() => ctx.eval("x = 100")).toThrow();
});

test("registerGlobalProperty", () => {
  const ctx = new Context();
  ctx.registerGlobalProperty("myNum", 1234, 7); // ALL attributes
  expect(ctx.eval("myNum")).toBe(1234);
});

test("registerGlobalCallable and registerGlobalBuiltinCallable", () => {
  const ctx = new Context();

  ctx.registerGlobalCallable("add", 2, (_thisArg, a, b) => {
    return (a as number) + (b as number);
  });

  expect(ctx.eval("add(5, 7)")).toBe(12);

  ctx.registerGlobalBuiltinCallable("multiply", 2, (_thisArg, a, b) => {
    return (a as number) * (b as number);
  });

  expect(ctx.eval("multiply(3, 4)")).toBe(12);
});

test("globalKeys and globalObject", () => {
  const ctx = new Context();
  ctx.registerGlobalProperty("customProp", "hello", 7);
  const keys = ctx.globalKeys();
  expect(keys).toContain("customProp");

  const glob = ctx.globalObject() as Record<string, unknown>;
  expect(glob).toBeDefined();
  expect(glob.customProp).toBe("hello");
});

test("runtimeLimits and setRuntimeLimits", () => {
  const ctx = new Context();
  const limits = ctx.runtimeLimits();
  expect(limits).toBeDefined();

  ctx.setRuntimeLimits({
    stackSize: 1024 * 1024,
    loopIterationLimit: 5000,
  });

  const newLimits = ctx.runtimeLimits();
  expect(newLimits.stackSize).toBe(1024 * 1024);
  expect(newLimits.loopIterationLimit).toBe(5000);
});

test("optimizerOptions and setOptimizerOptions", () => {
  const ctx = new Context();
  const opts = ctx.optimizerOptions();
  expect(opts).toBeDefined();

  ctx.setOptimizerOptions({
    constantFolding: true,
    deadCodeElimination: true,
  });

  const newOpts = ctx.optimizerOptions();
  expect(newOpts.constantFolding).toBe(true);
  expect(newOpts.deadCodeElimination).toBe(true);
});

test("stackTrace", () => {
  const ctx = new Context();
  const trace = ctx.stackTrace();
  expect(Array.isArray(trace)).toBe(true);
});

test("enqueueJob and runJobs", () => {
  const ctx = new Context();
  let ran = false;
  ctx.enqueueJob(() => {
    ran = true;
  });
  expect(ran).toBe(false);

  ctx.runJobs();
  expect(ran).toBe(true);
});

test("Realms API on Context", () => {
  const ctx = new Context();
  const realm1 = ctx.createRealm();
  expect(realm1.id).toBeDefined();
  expect(realm1.ctxId).toBeDefined();

  realmRegisterProperty(realm1, "realmVal", 999, true, true, true);
  expect(realmEval(realm1, "realmVal")).toBe(999);

  const globObj = realmGlobalObject(realm1);
  expect(globObj).toBeDefined();

  const globThis = realmGlobalThis(realm1);
  expect(globThis).toBeDefined();

  const keys = realmGlobalKeys(realm1);
  expect(keys).toContain("realmVal");

  realmRunJobs(realm1);

  const curRealm = ctx.realm();
  expect(curRealm.id).toBeDefined();

  const prevRealm = ctx.enterRealm(realm1);
  expect(prevRealm.id).toBeDefined();
});

test("Host data store on Context", () => {
  const ctx = new Context();
  ctx.insertData("userData", { name: "Alice", id: 42 });
  expect(ctx.hasData("userData")).toBe(true);
  expect(ctx.getData("userData")).toEqual({ name: "Alice", id: 42 });

  const removed = ctx.removeData("userData");
  expect(removed).toEqual({ name: "Alice", id: 42 });
  expect(ctx.hasData("userData")).toBe(false);
});

test("clearKeptObjects and dispose", () => {
  const ctx = new Context();
  ctx.clearKeptObjects();
  ctx.dispose();
  expect(() => ctx.eval("1 + 1")).toThrow();
});
