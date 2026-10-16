const assert = require("node:assert");
const test = require("node:test");
const { add, Engine, Parser } = require("./dist/index.js");

test("add", () => {
  assert.strictEqual(add(1, 2), 3);
});

test("Engine", () => {
  const engine = new Engine();
  assert.strictEqual(engine.is_job_pending(), false);
  assert.strictEqual(engine.execPendingJob(), false);
});
