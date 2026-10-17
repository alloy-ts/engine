/**
 * Tests for the embedded QuickJS `Context` and `Engine` bindings.
 *
 *   - `Context` — `src/context.rs`  (embedded QuickJS realm)
 *   - `Engine`  — `src/engine.rs`   (embedded QuickJS engine; mints contexts)
 *
 * Per the Rust bindings, both use real constructors, so instances are made with
 * `new Context()` / `new Engine()`.
 */
import { describe, expect, test } from "vite-plus/test";
import { Context, Engine } from "./main.ts";

describe("napi: Context", () => {
  test("new Context() evaluates expressions via eval", () => {
    const ctx = new Context();
    expect(ctx.eval("6 * 7")).toBe(42);
    expect(ctx.eval("'hi' + '!'")).toBe("hi!");
  });

  test("eval returns structured values", () => {
    const ctx = new Context();
    expect(ctx.eval("({ a: 1, b: [2, 3] })")).toEqual({ a: 1, b: [2, 3] });
  });

  test("global_keys returns a non-empty array of strings", () => {
    const ctx = new Context();
    const keys = ctx.global_keys();
    expect(Array.isArray(keys)).toBe(true);
    expect(keys.every((k) => typeof k === "string")).toBe(true);
    // QuickJS owns a few globals (e.g. `performance`).
    expect(keys.length).toBeGreaterThan(0);
  });

  test("eval_module returns the module namespace", () => {
    const ctx = new Context();
    const ns = ctx.eval_module("export const x = 5; export const arr = [1, 2, 3];") as {
      x: number;
      arr: number[];
    };
    expect(ns.x).toBe(5);
    expect(ns.arr).toEqual([1, 2, 3]);
  });

  test("a standalone Context has its own private engine (realm isolation)", () => {
    const a = new Context();
    const b = new Context();
    a.eval("globalThis.tag = 'a';");
    // Realms are isolated: `b` does not see `a`'s global.
    expect(b.eval("typeof globalThis.tag")).toBe("undefined");
  });

  test("Context static constructors (fromRaw, base, custom, full, builder)", () => {
    const fromRawCtx = Context.fromRaw();
    expect(fromRawCtx).toBeDefined();
    expect(fromRawCtx.eval("1 + 1")).toBe(2);

    const baseCtx = Context.base();
    expect(baseCtx).toBeDefined();

    const customCtx = Context.custom();
    expect(customCtx).toBeDefined();

    const fullCtx = Context.full();
    expect(fullCtx).toBeDefined();

    const builderCtx = Context.builder();
    expect(builderCtx).toBeDefined();
  });

  test("Context prototype methods (asRaw, getEnginePtr, Engine, with)", () => {
    const ctx = new Context();
    expect(typeof ctx.asRaw()).toBe("string");
    expect(typeof ctx.getEnginePtr()).toBe("string");

    const engine = ctx.Engine();
    expect(engine).toBeInstanceOf(Engine);

    const withRes = ctx.with(() => "hello from with");
    expect(withRes).toBe("hello from with");
  });
});

describe("napi: Engine + Context", () => {
  test("new Engine() and create_context mint a working Context", () => {
    const engine = new Engine();
    const ctx = engine.create_context();
    expect(ctx.eval("6 * 7")).toBe(42);
  });

  test("multiple contexts from one engine are independent realms", () => {
    const engine = new Engine();
    const a = engine.create_context();
    const b = engine.create_context();
    a.eval("globalThis.shared = 99;");
    expect(a.eval("globalThis.shared")).toBe(99);
    expect(b.eval("typeof globalThis.shared")).toBe("undefined");
  });

  test("engine.set_info does not throw", () => {
    const engine = new Engine();
    engine.set_info("test-engine");
  });

  test("engine.weak() returns a WeakEngine whose try_ref upgrades back to an Engine", () => {
    const engine = new Engine();
    const weak = engine.weak();
    const upgraded = weak.try_ref();
    expect(upgraded).toBeInstanceOf(Engine);
  });
});
