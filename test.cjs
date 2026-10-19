const assert = require("node:assert");
const { Context, Engine } = require("./dist/index.js");

const engine = new Engine();
const ctx = engine.createContext();
assert.strictEqual(ctx.eval("1 + 1"), 2);
assert.strictEqual(ctx.eval("console.log(3 + 3.33333)"), null);
ctx.dispose();
console.log("CJS native binding test passed!");
