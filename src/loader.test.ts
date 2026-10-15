/**
 * Tests for the host-provided QuickJS module loader (`src/loader.rs`).
 *
 * The `Loader` class is a host-driven `Resolver`/`Loader`: JS registers module
 * sources, and `import` statements inside an embedded context resolve against
 * that registry. It is attached to a `Engine` either at construction
 * (`new Engine(loader)`) or afterwards (`rt.set_loader(loader)`).
 *
 * Per the Rust bindings, `eval_module` (not `evalModule`) is used to evaluate a
 * module, and it returns the module's namespace object.
 */
import { describe, expect, test } from "vite-plus/test";
import { Context, Loader, Engine } from "./main.ts";

describe("napi: Loader (registry)", () => {
  test("add / has / keys / remove / clear", () => {
    const loader = new Loader();
    expect(loader.has("math")).toBe(false);

    loader.add("math", "export const x = 1;");
    expect(loader.has("math")).toBe(true);
    expect(loader.keys()).toContain("math");

    // add replaces existing source.
    loader.add("math", "export const x = 2;");
    expect(loader.has("math")).toBe(true);

    expect(loader.remove("math")).toBe(true);
    expect(loader.remove("math")).toBe(false);
    expect(loader.has("math")).toBe(false);

    loader.add("a", "export default 1;");
    loader.add("b", "export default 2;");
    expect(loader.keys().sort()).toEqual(["a", "b"]);
    loader.clear();
    expect(loader.keys()).toEqual([]);
  });
});

describe("napi: Engine + Loader (module resolution)", () => {
  test("imports a bare specifier from the loader (constructor form)", () => {
    const loader = new Loader();
    loader.add("math", "export const add = (a, b) => a + b;");

    const rt = new Engine(loader);
    const ctx = rt.create_context();

    ctx.eval_module('import { add } from "math"; globalThis.sum = add(2, 3);');
    expect(ctx.eval("globalThis.sum")).toBe(5);
  });

  test("imports a bare specifier via set_loader (post-construction form)", () => {
    const loader = new Loader();
    loader.add("greet", 'export const hello = () => "hi";');

    const rt = new Engine();
    rt.set_loader(loader);
    const ctx = rt.create_context();

    ctx.eval_module('import { hello } from "greet"; globalThis.msg = hello();');
    expect(ctx.eval("globalThis.msg")).toBe("hi");
  });

  test("resolves relative imports across a module graph", () => {
    const loader = new Loader();
    loader.add(
      "pkg/index",
      'import { value } from "./util"; export const doubled = value * 2;',
    );
    loader.add("pkg/util", "export const value = 21;");

    const rt = new Engine(loader);
    const ctx = rt.create_context();

    ctx.eval_module('import { doubled } from "pkg/index"; globalThis.out = doubled;');
    expect(ctx.eval("globalThis.out")).toBe(42);
  });

  test("throws when an imported module is missing", () => {
    const rt = new Engine(new Loader());
    const ctx = rt.create_context();
    expect(() => ctx.eval_module('import { x } from "does-not-exist";')).toThrow();
  });

  test("a loader added to one runtime does not leak to a standalone Context", () => {
    const loader = new Loader();
    loader.add("secret", "export const v = 1;");
    const rt = new Engine(loader);
    expect(rt.create_context().eval("1")).toBe(1);

    // A standalone Context has its own private runtime (no loader).
    const standalone = new Context();
    expect(() => standalone.eval_module('import { v } from "secret";')).toThrow();
  });
});
