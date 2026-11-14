// A runnable tour of the Engine/Context API against the built addon.
// Run `npm run build` first, then `npm run example`.
import assert from "node:assert/strict";
import { Engine } from "../dist/index.js";

const engine = new Engine();
const context = engine.createContext();

// Completion values come back as host primitives.
assert.equal(context.eval("1 + 2 * 3"), 7);
assert.equal(context.eval("'a' + 1"), "a1");
assert.equal(context.eval("1 < 2"), true);
assert.ok(Number.isNaN(context.eval("0 / 0")));
assert.equal(context.eval("1 / 0"), Infinity);

// Globals round-trip between the host and the Realm, -0 included.
context.set("negativeZero", -0);
assert.ok(Object.is(context.get("negativeZero"), -0));
assert.equal(context.eval("let x = 2; x * 21"), 42);

// Throw completions become host errors of the matching type.
assert.throws(() => context.eval("missing"), ReferenceError);

// Each Context has its own Realm and globals.
const other = engine.createContext();
assert.throws(() => other.eval("x"), ReferenceError);

// Disposing is idempotent; a disposed Context refuses further work.
context.dispose();
context.dispose();
assert.equal(context.disposed, true);
assert.throws(() => context.eval("1"));
other.dispose();

console.log("examples/engine.ts: all assertions passed");
