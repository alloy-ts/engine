import { describe, expect, test } from "vite-plus/test";
import { Context, Engine } from "./main.ts";

describe("napi: Context", () => {
  test("Context static constructors and prototype methods", () => {
    const engine = new Engine();

    const fullCtx = Context.full(engine);
    expect(fullCtx).toBeDefined();

    const baseCtx = Context.base(engine);
    expect(baseCtx).toBeDefined();

    const customCtx = Context.custom(engine);
    expect(customCtx).toBeDefined();

    const builderCtx = Context.builder(engine);
    expect(builderCtx).toBeDefined();

    const rawPtr = fullCtx.asRaw();
    expect(typeof rawPtr).toBe("bigint");

    const fromRawCtx = Context.fromRaw(engine, rawPtr);
    expect(fromRawCtx).toBeDefined();

    const enginePtr = fullCtx.getEnginePtr();
    expect(typeof enginePtr).toBe("bigint");

    const innerEngine = fullCtx.Engine();
    expect(innerEngine).toBeDefined();

    const withRes = fullCtx.with(() => "result-from-with");
    expect(withRes).toBe("result-from-with");

    expect(fullCtx.eval("2 + 3")).toBe("5");
  });
});
