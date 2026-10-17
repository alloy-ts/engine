import { expect, test } from "vite-plus/test";
import { Context, Engine as JsEngine } from "./main.ts";

test("Context static constructor methods exist and work", () => {
  expect(typeof Context.fromRaw).toBe("function");
  expect(typeof Context.base).toBe("function");
  expect(typeof Context.custom).toBe("function");
  expect(typeof Context.full).toBe("function");
  expect(typeof Context.builder).toBe("function");

  const ctxFull = Context.full();
  expect(ctxFull).toBeInstanceOf(Context);
  expect(ctxFull.eval("1 + 1")).toBe(2);

  const ctxBase = Context.base();
  expect(ctxBase).toBeInstanceOf(Context);

  const ctxCustom = Context.custom();
  expect(ctxCustom).toBeInstanceOf(Context);

  const ctxBuilder = Context.builder();
  expect(ctxBuilder).toBeInstanceOf(Context);

  const engine = new JsEngine();
  const ctxWithEngine = Context.full(engine);
  expect(ctxWithEngine).toBeInstanceOf(Context);

  const rawPtr = ctxFull.asRaw();
  expect(typeof rawPtr).toBe("bigint");

  const ctxFromRaw = Context.fromRaw(rawPtr, ctxFull.Engine());
  expect(ctxFromRaw).toBeInstanceOf(Context);
});

test("Context prototype methods exist and work", () => {
  expect(typeof Context.prototype.asRaw).toBe("function");
  expect(typeof Context.prototype.getEnginePtr).toBe("function");
  expect(typeof Context.prototype.Engine).toBe("function");
  expect(typeof Context.prototype.with).toBe("function");

  const ctx = new Context();

  const asRawVal = ctx.asRaw();
  expect(typeof asRawVal).toBe("bigint");

  const enginePtr = ctx.getEnginePtr();
  expect(typeof enginePtr).toBe("bigint");

  const engineObj = ctx.Engine();
  expect(engineObj).toBeInstanceOf(JsEngine);

  let called = false;
  ctx.with((innerCtx: any) => {
    called = true;
    expect(innerCtx.eval("2 * 3")).toBe(6);
  });
  expect(called).toBe(true);
});
