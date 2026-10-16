import { describe, expect, test } from "vite-plus/test";
import { BuiltinLoader, BuiltinResolver, Engine, FileResolver, ScriptLoader } from "./main.ts";

describe("napi: Loader", () => {
  test("constructs loader and resolver wrappers", () => {
    const loader = new BuiltinLoader();
    const resolver = new BuiltinResolver();
    const scriptLoader = new ScriptLoader();
    const fileResolver = new FileResolver();

    expect(loader).toBeDefined();
    expect(resolver).toBeDefined();
    expect(scriptLoader).toBeDefined();
    expect(fileResolver).toBeDefined();
  });

  test("sets loader and resolver on Engine", () => {
    const engine = new Engine();
    const loader = new BuiltinLoader();
    const resolver = new BuiltinResolver();

    engine.setLoader(resolver, loader);
  });
});
