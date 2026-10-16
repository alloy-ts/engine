import { describe, expect, test } from "vite-plus/test";
import { Runtime } from "./main.ts";

describe("napi: Runtime", () => {
  test("constructs Runtime and configures properties", () => {
    const rt = new Runtime();
    expect(rt).toBeDefined();

    rt.setInfo("test-rt");
    rt.setMemoryLimit(1024 * 1024);
    rt.setMaxStackSize(256 * 1024);
    rt.setGcThreshold(64 * 1024);
    rt.runGc();

    expect(rt.isJobPending()).toBe(false);
  });
});
