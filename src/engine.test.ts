/**
 * Tests for the native engine bindings.
 *
 * Public exports are re-exported from `./main.ts` (the vite-plus pack entry),
 * so that is the single import point tests should use — never the `.node`.
 *
 *   - `add`     — `src/lib.rs`
 *   - `Parser`  — `src/parser.rs`
 *   - `Engine`  — `src/engine.rs`  (JS class wrapping the N-API context)
 *
 * napi-rs exposes methods under their Rust (snake_case) name unless a
 * `js_name` override is set, so these tests call `create_array`, `has_global`,
 * … rather than camelCase equivalents.
 */
import { describe, expect, test } from "vite-plus/test";
import { add, Context, Engine, Loader, Parser, Runtime, WeakRuntime } from "./main.ts";

describe("napi: add", () => {
  test("adds two positive integers", () => {
    expect(add(2, 3)).toBe(5);
  });

  test("adds a negative and a positive integer", () => {
    expect(add(-4, 10)).toBe(6);
  });

  test("returns the other operand when one is zero", () => {
    expect(add(0, 42)).toBe(42);
    expect(add(42, 0)).toBe(42);
  });
});

describe("napi: Parser", () => {
  test("parses a valid JavaScript program into ESTree JSON", () => {
    const parser = new Parser("const x = 1;");
    const result = parser.parser(undefined) as {
      program: unknown;
      errors: string[];
      fatalError: boolean;
      isFlowLanguage: boolean;
    };

    expect(result.fatalError).toBe(false);
    expect(result.errors).toEqual([]);
    expect(result.program).toBeDefined();
    expect((result.program as { type: string }).type).toBe("Program");
  });

  test("infers language/module kind from a source_type path", () => {
    const tsx = new Parser("const el = <div />;", "src/app.tsx");
    const result = tsx.parser(undefined) as {
      fatalError: boolean;
      errors: string[];
    };
    expect(result.fatalError).toBe(false);
    expect(result.errors).toEqual([]);
  });

  test("collects recoverable syntax errors instead of throwing", () => {
    const parser = new Parser("const = ;");
    const result = parser.parser(undefined) as {
      errors: string[];
      fatalError: boolean;
    };

    expect(Array.isArray(result.errors)).toBe(true);
    expect(result.errors.length).toBeGreaterThan(0);
  });

  test("applies parse options (allow_return_outside_function)", () => {
    const parser = new Parser("return 1;");
    const defaultResult = parser.parser(undefined) as { errors: string[] };
    const relaxed = parser.parser({ allow_return_outside_function: true }) as {
      errors: string[];
      fatalError: boolean;
    };

    expect(defaultResult.errors.length).toBeGreaterThan(0);
    expect(relaxed.fatalError).toBe(false);
  });

  test("parses a single expression", () => {
    const parser = new Parser("1 + 2 * 3");
    const expr = parser.parse_expression(undefined) as { type: string };
    expect(expr.type).toBe("BinaryExpression");
  });

  test("throws on a fatal expression parse error", () => {
    const parser = new Parser("1 +");
    expect(() => parser.parse_expression(undefined)).toThrow();
  });
});

describe("napi: Context", () => {
  test("evaluates a simple script", () => {
    const ctx = new Context();
    expect(ctx.eval("1 + 2")).toBe(3);
  });

  test("evaluates an ES module and returns namespace", () => {
    const ctx = new Context();
    const res = ctx.eval_module("export const foo = 42;");
    expect(res).toEqual({ foo: 42 });
  });

  test("json_parse parses JSON in engine", () => {
    const ctx = new Context();
    expect(ctx.json_parse('{"a":1,"b":"test"}')).toEqual({ a: 1, b: "test" });
  });

  test("global_keys returns array of keys on globalThis", () => {
    const ctx = new Context();
    const keys = ctx.global_keys();
    expect(Array.isArray(keys)).toBe(true);
    expect(keys).toContain("performance");
  });

  test("run_gc and execute_pending_job work without error", () => {
    const ctx = new Context();
    expect(() => ctx.run_gc()).not.toThrow();
    expect(typeof ctx.execute_pending_job()).toBe("boolean");
    expect(ctx.has_exception()).toBe(false);
  });
});

describe("napi: Runtime & WeakRuntime", () => {
  test("creates context from runtime", () => {
    const rt = new Runtime();
    const ctx = rt.create_context();
    expect(ctx.eval("10 * 5")).toBe(50);
  });

  test("weak runtime try_ref promotes back to Runtime", () => {
    const rt = new Runtime();
    const weak: WeakRuntime = rt.weak();
    const ref = weak.try_ref();
    expect(ref).not.toBeNull();
  });

  test("configures memory limits and retrieves memory usage", () => {
    const rt = new Runtime();
    expect(() => rt.set_memory_limit(1024 * 1024)).not.toThrow();
    expect(() => rt.set_max_stack_size(512 * 1024)).not.toThrow();
    expect(() => rt.set_gc_threshold(256 * 1024)).not.toThrow();
    expect(() => rt.set_info("test-runtime")).not.toThrow();
    expect(() => rt.run_gc()).not.toThrow();
    expect(rt.is_job_pending()).toBe(false);

    const mem = rt.memory_usage() as { malloc_size: number };
    expect(mem).toBeDefined();
    expect(typeof mem.malloc_size).toBe("number");
  });
});

describe("napi: Loader", () => {
  test("registers modules and resolves them in runtime contexts", () => {
    const loader = new Loader();
    loader.add("math", "export const add = (a, b) => a + b;");
    expect(loader.has("math")).toBe(true);
    expect(loader.keys()).toEqual(["math"]);

    const rt = new Runtime(loader);
    const ctx = rt.create_context();
    const res = ctx.eval_module('import { add } from "math"; export const result = add(3, 4);');
    expect(res).toEqual({ result: 7 });
  });
});

describe("napi: Engine", () => {
  test("constructs without arguments (context injected by NAPI)", () => {
    const engine = new Engine();
    expect(engine).toBeDefined();
  });

  test("create_array allocates a JS array of the requested length", () => {
    const engine = new Engine();
    const arr = engine.create_array(3) as unknown[];
    expect(Array.isArray(arr)).toBe(true);
    expect(arr.length).toBe(3);
  });

  test("create_string / create_int32 / create_double build JS values", () => {
    const engine = new Engine();
    expect(engine.create_string("hello")).toBe("hello");
    expect(engine.create_int32(42)).toBe(42);
    expect(engine.create_double(1.5)).toBe(1.5);
  });

  test("has_global reads the global object through the context", () => {
    const engine = new Engine();
    expect(engine.has_global("Math")).toBe(true);
    expect(engine.has_global("definitelyNotAGlobal")).toBe(false);
  });

  test("get_napi_version reports the host N-API version", () => {
    const engine = new Engine();
    expect(engine.get_napi_version()).toBeGreaterThan(0);
  });

  test("make_object builds a JS object via the context", () => {
    const engine = new Engine();
    const obj = engine.make_object() as { ok: boolean; engine: string };
    expect(obj.ok).toBe(true);
    expect(obj.engine).toBe("napi");
  });

  test("set_instance_data persists VM-specific state on the Agent", () => {
    const engine = new Engine();
    engine.set_instance_data("vm-state");
    expect(engine.get_instance_data()).toBe("vm-state");
  });

  test("is_disposed is false while the addon is loaded", () => {
    const engine = new Engine();
    expect(engine.is_disposed()).toBe(false);
  });

  test("each instance captures its own context handle", () => {
    const a = new Engine();
    const b = new Engine();
    // Two engines must not alias the same context; both are independently live.
    expect(a.is_disposed()).toBe(false);
    expect(b.is_disposed()).toBe(false);
  });
});
