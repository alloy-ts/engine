const { test } = require("node:test");
const assert = require("node:assert");
const engine = require("./dist/index.js");

void test("addon exports", () => {
  assert.strictEqual(typeof engine.add, "function");
  assert.strictEqual(typeof engine.Parser, "function");
  assert.strictEqual(typeof engine.Engine, "function");
});
