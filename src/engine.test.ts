import { expect, test } from "vite-plus/test";
import { Context, Engine } from "./main.ts";

test("Engine constructor and configuration methods", () => {
  const engine = new Engine();
  expect(engine).toBeDefined();

  expect(typeof engine.set_memory_limit).toBe("function");
  expect(typeof engine.set_max_stack_size).toBe("function");
  expect(typeof engine.set_gc_threshold).toBe("function");
  expect(typeof engine.set_info).toBe("function");
  expect(typeof engine.run_gc).toBe("function");
  expect(typeof engine.memory_usage).toBe("function");
  expect(typeof engine.is_job_pending).toBe("function");
  expect(typeof engine.execute_pending_job).toBe("function");
  expect(typeof engine.weak).toBe("function");

  engine.set_memory_limit(1024 * 1024 * 64);
  engine.set_max_stack_size(256 * 1024);
  engine.set_gc_threshold(1024 * 1024 * 16);
  engine.set_info("test-engine");
  engine.run_gc();

  const mem = engine.memory_usage();
  expect(mem).toBeDefined();

  const isPending = engine.is_job_pending();
  expect(typeof isPending).toBe("boolean");

  const weak = engine.weak();
  expect(weak).toBeDefined();
  expect(typeof weak.try_ref).toBe("function");
});

test("Engine host helper methods", () => {
  const engine = new Engine();
  expect(engine.is_disposed()).toBe(false);

  const arr = engine.create_array(5);
  expect(arr).toBeDefined();

  const str = engine.create_string("hello");
  expect(str).toBe("hello");

  const num = engine.create_int32(42);
  expect(num).toBe(42);

  const dbl = engine.create_double(3.14159);
  expect(dbl).toBe(3.14159);

  const napiVer = engine.get_napi_version();
  expect(typeof napiVer).toBe("number");

  const obj = engine.make_object();
  expect(obj).toBeDefined();
});
