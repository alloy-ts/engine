const test = require("node:test");
const assert = require("node:assert");
const native = require("./dist/index.js");

test("native addon exports Context, Runtime, Engine, Parser, add", () => {
  assert.strictEqual(typeof native.add, "function");
  assert.strictEqual(native.add(2, 3), 5);
  assert.strictEqual(typeof native.Context, "function");
  assert.strictEqual(typeof native.Runtime, "function");
  assert.strictEqual(typeof native.Engine, "function");
  assert.strictEqual(typeof native.Parser, "function");

  const ctx = new native.Context();
  assert.ok(ctx);
  assert.strictEqual(ctx.eval("1 + 1"), 2);
});
