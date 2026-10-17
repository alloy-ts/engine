import { expect, test } from "vite-plus/test";
import {
  Context,
  realmEval,
  realmGlobalKeys,
  realmRegisterGlobalProperty,
  realmRunJobs,
} from "./main.ts";

test("Context default constructor and console.log eval", () => {
  const ctx = new Context();
  const res = ctx.eval("console.log(3 + 3.33333)");
  expect(res).toBeNull();

  const numRes = ctx.eval("3 + 3.33333");
  expect(numRes).toBe(6.33333);
});

test("Context builder pattern", () => {
  const builder = Context.builder();
  builder.canBlock(false);
  const ctx = builder.build();
  expect(ctx.eval("10 * 10")).toBe(100);
});

test("Context script evaluation", () => {
  const ctx = new Context();
  expect(ctx.eval("'Hello, ' + 'Alloy!'")).toBe("Hello, Alloy!");
  expect(ctx.eval("function add(a, b) { return a + b; }; add(5, 7);")).toBe(12);
});

test("Context global keys and global object", () => {
  const ctx = new Context();
  const keys = ctx.globalKeys();
  expect(Array.isArray(keys)).toBe(true);
  expect(keys.includes("console")).toBe(true);

  const globalObj = ctx.globalObject();
  expect(globalObj).not.toBeNull();
});

test("Context global property registration", () => {
  const ctx = new Context();
  ctx.registerGlobalProperty("myConfig", { env: "test", port: 8080 }, 7);
  expect(ctx.eval("myConfig.env")).toBe("test");
  expect(ctx.eval("myConfig.port")).toBe(8080);
});

test("Context global callable registration", () => {
  const ctx = new Context();
  let called = false;
  let receivedArgs: unknown[] = [];

  ctx.registerGlobalCallable("customHostFunc", 2, (a, b) => {
    called = true;
    receivedArgs = [a, b];
    return 42;
  });

  const res = ctx.eval("customHostFunc('a', 'b')");
  expect(called).toBe(true);
  expect(receivedArgs).toEqual(["a", "b"]);
  expect(res).toBe(42);
});

test("Context global builtin callable registration", () => {
  const ctx = new Context();
  ctx.registerGlobalBuiltinCallable("builtinFunc", 1, (val) => {
    return `echo:${val}`;
  });

  expect(ctx.eval("builtinFunc('hello')")).toBe("echo:hello");
  expect(() => ctx.eval("new builtinFunc('hello')")).toThrow();
});

test("Context realm management", () => {
  const ctx = new Context();
  const mainRealm = ctx.realm();
  expect(mainRealm.id).toBeDefined();
  expect(mainRealm.ctxId).toBeDefined();

  const newRealm = ctx.createRealm();
  expect(newRealm.id).not.toBe(mainRealm.id);

  const prevRealm = ctx.enterRealm(newRealm);
  expect(prevRealm.id).toBe(mainRealm.id);

  ctx.eval("globalThis.realmMarker = 'secondary';");
  expect(ctx.eval("globalThis.realmMarker")).toBe("secondary");

  ctx.enterRealm(prevRealm);
  expect(ctx.eval("globalThis.realmMarker")).toBeNull();
});

test("Realm helper free functions", () => {
  const ctx = new Context();
  const secondaryRealm = ctx.createRealm();

  realmRegisterGlobalProperty(secondaryRealm, "realmVar", 999);
  const evalRes = realmEval(secondaryRealm, "realmVar * 2");
  expect(evalRes).toBe(1998);

  const keys = realmGlobalKeys(secondaryRealm);
  expect(keys.includes("realmVar")).toBe(true);

  realmRunJobs(secondaryRealm);
});

test("Context job queueing and execution", () => {
  const ctx = new Context();
  let jobRan = false;

  ctx.enqueueJob(() => {
    jobRan = true;
  });

  expect(jobRan).toBe(false);
  ctx.runJobs();
  expect(jobRan).toBe(true);
});

test("Context runtime limits", () => {
  const ctx = new Context();
  const limits = ctx.runtimeLimits();
  expect(limits.stackSize).toBeDefined();

  ctx.setRuntimeLimits({
    stackSize: 256 * 1024,
    recursionLimit: 500,
  });

  const updatedLimits = ctx.runtimeLimits();
  expect(updatedLimits.stackSize).toBe(256 * 1024);
  expect(updatedLimits.recursionLimit).toBe(500);
});

test("Context optimizer options", () => {
  const ctx = new Context();
  const opts = ctx.optimizerOptions();
  expect(opts.constantFolding).toBeDefined();

  ctx.setOptimizerOptions({
    constantFolding: true,
    deadCodeElimination: false,
  });

  const updatedOpts = ctx.optimizerOptions();
  expect(updatedOpts.constantFolding).toBe(true);
  expect(updatedOpts.deadCodeElimination).toBe(false);
});

test("Context stack trace", () => {
  const ctx = new Context();
  const trace = ctx.stackTrace();
  expect(Array.isArray(trace)).toBe(true);
});

test("Context clear kept objects", () => {
  const ctx = new Context();
  expect(() => ctx.clearKeptObjects()).not.toThrow();
});

test("Context host-defined data store", () => {
  const ctx = new Context();
  expect(ctx.hasData("key1")).toBe(false);

  ctx.insertData("key1", { token: "abc" });
  expect(ctx.hasData("key1")).toBe(true);
  expect(ctx.getData("key1")).toEqual({ token: "abc" });

  const removed = ctx.removeData("key1");
  expect(removed).toEqual({ token: "abc" });
  expect(ctx.hasData("key1")).toBe(false);
});

test("Context disposal", () => {
  const ctx = new Context();
  ctx.dispose();
  expect(() => ctx.eval("1 + 1")).toThrow("Context has been disposed");
});
