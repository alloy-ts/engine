import assert from "node:assert/strict";

// Run after building: node --import ./dist/index.js examples/boolean.ts

interface ExtendedBooleanConstructor extends BooleanConstructor {
  of?(value: boolean): boolean | Boolean;
}

interface ExtendedBoolean extends Boolean {
  toInteger?(): number | bigint;
}

const ExtendedBoolean = globalThis.Boolean as ExtendedBooleanConstructor;

// The original callable and boxed forms continue to work.
assert.equal(ExtendedBoolean(1), true);
assert.equal(ExtendedBoolean(0), false);

const boxed = new ExtendedBoolean(true) as ExtendedBoolean;
assert.equal(boxed instanceof Boolean, true);
assert.equal(boxed.valueOf(), true);

// Future native extensions remain additive to the host constructor/prototype.
const extended = ExtendedBoolean.of?.(true) ?? boxed;
assert.equal(Boolean(extended), true);
assert.equal(boxed.toInteger?.() ?? Number(boxed.valueOf()), 1);

console.log({ value: boxed.valueOf() });
