const assert = require("node:assert");
const { test } = require("node:test");
const { Context, Engine, Script } = require("./dist/index.js");

test("CommonJS Context eval", () => {
  const ctx = new Context();
  assert.strictEqual(ctx.eval("1 + 1"), 2);
  assert.strictEqual(ctx.eval("3 + 3.33333"), 6.33333);
  ctx.dispose();
});

test("CommonJS Engine functionality", () => {
  const engine = new Engine();
  assert.strictEqual(engine.version(), "0.22.0");
  const ctx = engine.createContext();
  assert.strictEqual(ctx.eval("2 * 3"), 6);
  ctx.dispose();
});
