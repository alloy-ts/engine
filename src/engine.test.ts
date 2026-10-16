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
import { add, Atom, Compiler, Context, Engine, Loader, Parser, Runtime } from "./main.ts";

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

  test("Engine creates QuickJS contexts and executes code", () => {
    const engine = new Engine();
    const ctx = engine.create_context();
    expect(ctx.eval("2 + 3")).toBe(5);
  });

  test("Engine manages QuickJS memory limits and usage stats", () => {
    const engine = new Engine();
    engine.set_memory_limit(1024 * 1024 * 32);
    engine.set_max_stack_size(256 * 1024);
    engine.run_gc();

    const stats = engine.memory_usage() as { malloc_count: number };
    expect(stats).toBeDefined();
    expect(typeof stats.malloc_count).toBe("number");
  });

  test("Engine supports execPendingJob and is_job_pending", () => {
    const engine = new Engine();
    expect(engine.is_job_pending()).toBe(false);
    expect(engine.execPendingJob()).toBe(false);
    expect(engine.execute_pending_job()).toBe(false);
  });
});

describe("napi: Context", () => {
  test("eval computes expressions", () => {
    const ctx = Context.new();
    expect(ctx.eval("1 + 1")).toBe(2);
    expect(ctx.eval("'hello' + ' ' + 'world'")).toBe("hello world");
  });

  test("eval_module evaluates ES module exports", () => {
    const ctx = Context.new();
    const result = ctx.eval_module("export const a = 10; export const b = 20;") as {
      a: number;
      b: number;
    };
    expect(result.a).toBe(10);
    expect(result.b).toBe(20);
  });

  test("json_parse parses JSON string into JS value", () => {
    const ctx = Context.new();
    const parsed = ctx.json_parse('{"x": 42, "y": [1, 2, 3]}') as {
      x: number;
      y: number[];
    };
    expect(parsed.x).toBe(42);
    expect(parsed.y).toEqual([1, 2, 3]);
  });

  test("global_keys returns array of global property names", () => {
    const ctx = Context.new();
    const keys = ctx.global_keys();
    expect(Array.isArray(keys)).toBe(true);
    expect(keys).toContain("Math");
    expect(keys).toContain("Array");
  });

  test("run_gc and execute_pending_job run without error", () => {
    const ctx = Context.new();
    ctx.run_gc();
    expect(ctx.execute_pending_job()).toBe(false);
  });

  test("eval_with_filename evaluates script with source name", () => {
    const ctx = Context.new();
    const val = ctx.eval_with_filename("2 * 21", "test.js");
    expect(val).toBe(42);
  });

  test("compile validates script syntax", () => {
    const ctx = Context.new();
    expect(() => ctx.compile("const x = 1;", "test.js", true)).not.toThrow();
  });
});

describe("napi: Atom", () => {
  test("creates Atom from string, numbers, and bools", () => {
    const atomStr = new Atom("foo");
    expect(atomStr.to_string()).toBe("foo");
    expect(atomStr.to_value()).toBe("foo");

    const atomU32 = Atom.from_u32(123);
    expect(atomU32.to_string()).toBe("123");

    const atomI32 = Atom.from_i32(-456);
    expect(atomI32.to_string()).toBe("-456");

    const atomBool = Atom.from_bool(true);
    expect(atomBool.to_string()).toBe("true");

    const atomF64 = Atom.from_f64(3.14);
    expect(atomF64.to_string()).toBe("3.14");
  });
});

describe("napi: Compiler", () => {
  test("Compiler executes compilation and outputs AST string", () => {
    const compiler = new Compiler();
    const result = compiler.execute("const x = 1;", undefined, "index.js");
    expect(typeof result).toBe("string");
    expect(result).toContain("Program");
  });

  test("Compiler throws on syntax error", () => {
    const compiler = new Compiler();
    expect(() => compiler.execute("const =", undefined, "index.js")).toThrow();
  });
});

describe("napi: Runtime & Loader", () => {
  test("Runtime creates shared contexts", () => {
    const rt = new Runtime();
    const ctx1 = rt.create_context();
    const ctx2 = rt.create_context();

    expect(ctx1.eval("1 + 1")).toBe(2);
    expect(ctx2.eval("2 + 2")).toBe(4);
  });

  test("Runtime configures limits and memory_usage", () => {
    const rt = new Runtime();
    rt.set_memory_limit(1024 * 1024 * 16);
    rt.set_max_stack_size(256 * 1024);
    rt.set_gc_threshold(1024 * 64);
    rt.run_gc();

    const mem = rt.memory_usage() as { malloc_count: number };
    expect(mem).toBeDefined();
    expect(typeof mem.malloc_count).toBe("number");
  });

  test("WeakRuntime promotes to Runtime", () => {
    const rt = new Runtime();
    const weak = rt.weak();
    const promoted = weak.try_ref();
    expect(promoted).not.toBeNull();
  });

  test("Loader imports in-memory builtin modules", () => {
    const rt = new Runtime();
    const loader = new Loader();
    loader.add_builtin("custom_mod", "export const greeting = 'hello from loader';");
    rt.set_loader(loader);

    const ctx = rt.create_context();
    const mod = ctx.eval_module("import { greeting } from 'custom_mod'; export { greeting };") as {
      greeting: string;
    };

    expect(mod.greeting).toBe("hello from loader");
  });
});
