/**
 * Tests for the native engine bindings.
 *
 * Public exports are re-exported from `./main.ts` (the vite-plus pack entry),
 * so that is the single import point tests should use — never the `.node`.
 *
 *   - `add`     — `src/lib.rs`
 *   - `Parser`  — `src/parser.rs`
 *   - `Engine`  — `src/engine.rs`  (JS class wrapping N-API context + QuickJS Runtime)
 */
import { describe, expect, test } from "vite-plus/test";
import {
  BuiltinLoader,
  BuiltinResolver,
  FileResolver,
  ScriptLoader,
  add,
  Engine,
  Parser,
} from "./main.ts";

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

  test("create_context and create_base_context work", () => {
    const engine = new Engine();
    const ctx = engine.create_context();
    expect(ctx).toBeDefined();
    expect(ctx.eval("1 + 1")).toBe(2);

    const baseCtx = engine.create_base_context();
    expect(baseCtx).toBeDefined();
  });

  test("set_info sets description without throwing", () => {
    const engine = new Engine();
    expect(() => engine.set_info("test-engine")).not.toThrow();
  });

  test("memory limit, stack size, and GC threshold configuration", () => {
    const engine = new Engine();
    expect(() => engine.set_memory_limit(1024 * 1024)).not.toThrow();
    expect(() => engine.set_max_stack_size(256 * 1024)).not.toThrow();
    expect(() => engine.set_gc_threshold(512 * 1024)).not.toThrow();
    expect(() => engine.set_dump_flags(0)).not.toThrow();
  });

  test("run_gc and memory_usage", () => {
    const engine = new Engine();
    expect(() => engine.run_gc()).not.toThrow();
    const mem = engine.memory_usage();
    expect(mem).toBeDefined();
    expect(typeof mem.mallocSize).toBe("number");
    expect(typeof mem.memoryUsedSize).toBe("number");
  });

  test("job queue status and execution", () => {
    const engine = new Engine();
    expect(engine.is_job_pending()).toBe(false);
    expect(engine.execute_pending_job()).toBe(false);
  });

  test("module loaders setup", () => {
    const engine = new Engine();
    const fileResolver = new FileResolver();
    fileResolver.add_path("src");
    const scriptLoader = new ScriptLoader();
    expect(() => engine.set_file_loader(fileResolver, scriptLoader)).not.toThrow();

    const builtinResolver = new BuiltinResolver();
    builtinResolver.add_module("my-module");
    const builtinLoader = new BuiltinLoader();
    expect(() => engine.set_builtin_loader(builtinResolver, builtinLoader)).not.toThrow();
  });
});
