import { describe, expect, test } from "vite-plus/test";
import { BuiltinLoader, BuiltinResolver, FileResolver, Runtime, ScriptLoader } from "./main.ts";

describe("napi: Runtime", () => {
  test("constructs Runtime instance", () => {
    const rt = new Runtime();
    expect(rt).toBeDefined();
  });

  test("create_context and create_base_context work", () => {
    const rt = new Runtime();
    const ctx = rt.create_context();
    expect(ctx).toBeDefined();
    expect(ctx.eval("1 + 1")).toBe(2);

    const baseCtx = rt.create_base_context();
    expect(baseCtx).toBeDefined();
  });

  test("set_info sets runtime description without throwing", () => {
    const rt = new Runtime();
    expect(() => rt.set_info("test-runtime")).not.toThrow();
  });

  test("memory limit, stack size, and GC threshold configuration", () => {
    const rt = new Runtime();
    expect(() => rt.set_memory_limit(1024 * 1024)).not.toThrow();
    expect(() => rt.set_max_stack_size(256 * 1024)).not.toThrow();
    expect(() => rt.set_gc_threshold(512 * 1024)).not.toThrow();
    expect(() => rt.set_dump_flags(0)).not.toThrow();
  });

  test("run_gc and memory_usage", () => {
    const rt = new Runtime();
    expect(() => rt.run_gc()).not.toThrow();
    const mem = rt.memory_usage();
    expect(mem).toBeDefined();
    expect(typeof mem.mallocSize).toBe("number");
    expect(typeof mem.memoryUsedSize).toBe("number");
  });

  test("job queue status and execution", () => {
    const rt = new Runtime();
    expect(rt.is_job_pending()).toBe(false);
    expect(rt.execute_pending_job()).toBe(false);
  });

  test("module loaders setup", () => {
    const rt = new Runtime();
    const fileResolver = new FileResolver();
    fileResolver.add_path("src");
    const scriptLoader = new ScriptLoader();
    expect(() => rt.set_file_loader(fileResolver, scriptLoader)).not.toThrow();

    const builtinResolver = new BuiltinResolver();
    builtinResolver.add_module("my-module");
    const builtinLoader = new BuiltinLoader();
    expect(() => rt.set_builtin_loader(builtinResolver, builtinLoader)).not.toThrow();
  });
});
