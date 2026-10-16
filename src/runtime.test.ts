import { describe, expect, test } from "vite-plus/test";
import { Loader, Resolver, Runtime } from "./main.ts";

describe("napi: Runtime", () => {
  test("constructs Runtime", () => {
    const rt = new Runtime();
    expect(rt).toBeDefined();
  });

  test("configures memory limit, max stack size, and gc threshold", () => {
    const rt = new Runtime();
    expect(() => rt.setMemoryLimit(0xffff)).not.toThrow();
    expect(() => rt.setMaxStackSize(256 * 1024)).not.toThrow();
    expect(() => rt.setGcThreshold(0xff)).not.toThrow();
  });

  test("configures info and dump flags", () => {
    const rt = new Runtime();
    expect(() => rt.setInfo("test runtime")).not.toThrow();
    expect(() => rt.setDumpFlags(0)).not.toThrow();
  });

  test("setLoader configures resolver and loader", () => {
    const rt = new Runtime();
    const resolver = new Resolver();
    const loader = new Loader();
    expect(() => rt.setLoader(resolver, loader)).not.toThrow();
  });

  test("runGc executes garbage collection without error", () => {
    const rt = new Runtime();
    expect(() => rt.runGc()).not.toThrow();
  });

  test("isJobPending and executePendingJob report job status", () => {
    const rt = new Runtime();
    expect(rt.isJobPending()).toBe(false);
    expect(rt.executePendingJob()).toBe(false);
  });
});
