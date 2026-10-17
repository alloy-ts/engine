const test = require("node:test");
const assert = require("node:assert/strict");
const { Context, Engine } = require("./dist/index.js");

test("CJS Context evaluation", () => {
  const ctx = new Context();
  assert.equal(ctx.eval("1 + 1"), 2);
  assert.equal(ctx.eval("console.log(3 + 3.33333)"), null);
});

test("CJS Engine creation", () => {
  const engine = new Engine();
  assert.ok(engine);
  const ctx = engine.createContext();
  assert.equal(ctx.eval("10 * 10"), 100);
});
