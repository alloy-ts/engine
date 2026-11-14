import assert from "node:assert/strict";

// Run after building: node --import ./dist/index.js examples/string.ts

interface ExtendedStringConstructor extends StringConstructor {
  of?(value: unknown): string;
}

interface ExtendedString extends String {
  toBuffer?(encoding?: BufferEncoding): Buffer;
}

const ExtendedString = globalThis.String as ExtendedStringConstructor;

// Existing ECMAScript behavior remains intact.
assert.equal(ExtendedString(42), "42");
const boxed = new ExtendedString("alloy") as ExtendedString;
assert.equal(boxed instanceof String, true);
assert.equal(boxed.valueOf(), "alloy");
assert.equal(boxed.toUpperCase(), "ALLOY");

// Extension methods are additive and feature-detected until registered.
const value = ExtendedString.of?.("core") ?? ExtendedString("core");
assert.equal(value, "core");

const bytes = boxed.toBuffer?.("utf8") ?? Buffer.from(boxed.valueOf(), "utf8");
assert.equal(bytes.toString("utf8"), "alloy");

console.log({ value, bytes });
