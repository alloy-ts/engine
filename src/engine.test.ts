import { describe, expect, it } from "vitest";
import { Engine, Context } from "./main";

describe("Engine", () => {
  it("creates an engine instance and configures limits", () => {
    const engine = new Engine();
    expect(engine).toBeDefined();

    engine.setMemoryLimit(1024 * 1024 * 32);
    engine.setMaxStackSize(1024 * 512);
    engine.setGcThreshold(1024 * 256);
    engine.runGc();
  });

  it("handles job execution status and execPendingJob", () => {
    const engine = new Engine();
    expect(engine.isJobPending()).toBe(false);

    const executed = engine.execPendingJob();
    expect(executed).toBe(false);
  });

  it("works together with Context", () => {
    const engine = new Engine();
    const ctx = Context.full(engine);
    const result = ctx.eval("10 + 20");
    expect(result).toBe("30");
  });
});
