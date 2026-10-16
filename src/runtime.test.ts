import { describe, expect, test } from "vite-plus/test";
import {
  BuiltinLoader,
  BuiltinResolver,
  FileResolver,
  ModuleLoader,
  NativeLoader,
  Runtime,
  ScriptLoader,
} from "./main.ts";

describe("napi: Runtime & Loader", () => {
  test("constructs Runtime", () => {
    const rt = new Runtime();
    expect(rt).toBeDefined();
  });

  test("configures runtime info and limits", () => {
    const rt = new Runtime();
    expect(() => rt.setInfo("test-runtime")).not.toThrow();
    expect(() => rt.setMemoryLimit(1024 * 1024)).not.toThrow();
    expect(() => rt.setMaxStackSize(512 * 1024)).not.toThrow();
    expect(() => rt.setGcThreshold(64 * 1024)).not.toThrow();
    expect(() => rt.setDumpFlags(0)).not.toThrow();
  });

  test("runs garbage collection and gets memory usage", () => {
    const rt = new Runtime();
    expect(() => rt.runGc()).not.toThrow();
    const usage = rt.memoryUsage();
    expect(usage).toBeDefined();
    expect(typeof usage.memoryUsedSize).toBe("number");
  });

  test("creates weak reference and upgrades via try_ref", () => {
    const rt = new Runtime();
    const weak = rt.weak();
    expect(weak).toBeDefined();
    const ref = weak.tryRef();
    expect(ref).toBeDefined();
  });

  test("constructs loader and resolver types", () => {
    const builtinResolver = new BuiltinResolver();
    const builtinLoader = new BuiltinLoader();
    const fileResolver = new FileResolver();
    const scriptLoader = new ScriptLoader();
    const nativeLoader = new NativeLoader();
    const moduleLoader = new ModuleLoader();

    expect(builtinResolver).toBeDefined();
    expect(builtinLoader).toBeDefined();
    expect(fileResolver).toBeDefined();
    expect(scriptLoader).toBeDefined();
    expect(nativeLoader).toBeDefined();
    expect(moduleLoader).toBeDefined();
  });
});
