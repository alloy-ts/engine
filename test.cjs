const { Context, Engine } = require("./dist/index.js");
const assert = require("assert");

const ctx = new Context();
assert.strictEqual(ctx.eval("1 + 1"), 2);

const engine = new Engine();
assert.strictEqual(engine.backendName(), "boa");
assert.strictEqual(engine.eval("2 + 3"), 5);

console.log("CJS test passed!");
