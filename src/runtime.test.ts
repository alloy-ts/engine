import { describe, expect, test } from "vite-plus/test";
import { Runtime } from "./main.ts";

describe("napi: Runtime", () => {
  test("constructs a new Runtime", () => {
    const rt = new Runtime();
    expect(rt).toBeDefined();
  });

  test("configures memory and stack limits", () => {
    const rt = new Runtime();
    rt.setMemoryLimit(1024 * 1024);
    rt.setMaxStackSize(256 * 1024);
    rt.setGcThreshold(512 * 1024);
    rt.runGc();
    expect(rt.isJobPending()).toBe(false);
  });
});
