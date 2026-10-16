/**
 * Tests for the native engine bindings.
 *
 * Public exports are re-exported from `./main.ts` (the vite-plus pack entry),
 * so that is the single import point tests should use — never the `.node`.
 *
 *   - `add`      — `src/lib.rs`
 *   - `Parser`   — `src/parser.rs`
 *   - `Engine` / `JsEngine` — `src/engine.rs` (JS class wrapping the QuickJS runtime)
 */
import { describe, expect, test } from "vite-plus/test";
import { add, Context, Engine, JsEngine, Parser } from "./main.ts";

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

describe("napi: Engine & JsEngine", () => {
  test("constructs Engine and JsEngine without arguments", () => {
    const engine = new Engine();
    const jsEngine = new JsEngine();
    expect(engine).toBeDefined();
    expect(jsEngine).toBeDefined();
  });

  test("JsEngine is exported as an alias of Engine", () => {
    expect(JsEngine).toBe(Engine);
  });

  test("allows configuring engine info, memory limit, stack size, and gc threshold", () => {
    const engine = new JsEngine();
    expect(() => engine.set_info("test-engine")).not.toThrow();
    expect(() => engine.set_memory_limit(1024 * 1024)).not.toThrow();
    expect(() => engine.set_max_stack_size(256 * 1024)).not.toThrow();
    expect(() => engine.set_gc_threshold(64 * 1024)).not.toThrow();
  });

  test("run_gc executes garbage collection cycle without error", () => {
    const engine = new JsEngine();
    expect(() => engine.run_gc()).not.toThrow();
  });

  test("is_job_pending and execute_pending_job / execPendingJob", () => {
    const engine = new JsEngine();
    expect(engine.is_job_pending()).toBe(false);
    expect(engine.execute_pending_job()).toBe(false);
    expect(engine.execPendingJob()).toBe(false);
  });

  test("creates Context instances bound to the Engine", () => {
    const engine = new JsEngine();
    expect(() => Context.full(engine)).not.toThrow();
    expect(() => Context.base(engine)).not.toThrow();
  });
});
