const test = require("node:test");
const assert = require("node:assert/strict");

test("Smoke test loading engine via ESM import", async () => {
  const { Engine, Context } = await import("./dist/main.mjs");
  const engine = new Engine();
  assert.equal(engine.version(), "0.22.0");
  const ctx = new Context();
  assert.equal(ctx.eval("1 + 2"), 3);
  ctx.dispose();
});
