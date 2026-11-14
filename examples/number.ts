import assert from "node:assert/strict";

// Run after building: node --import ./dist/index.js examples/number.ts

interface IntegerStatics {
  cast(
    value: number | bigint,
    bits?: 8 | 16 | 32 | 64 | 128,
    unsigned?: boolean,
  ): bigint;
}

const { Integer } = globalThis as typeof globalThis & {
  Integer: IntegerStatics;
};

// The host Number global is not replaced by the addon.
assert.equal(Number("42"), 42);
assert.equal(new Number(42).valueOf(), 42);
assert.equal(Number.isInteger(42), true);
assert.equal(Number.isSafeInteger(Number.MAX_SAFE_INTEGER), true);

// Integer provides fixed-width and i128 behavior without changing Number.
assert.equal(Integer.cast(255, 8, true), 255n);
assert.equal(Integer.cast(255, 8, false), -1n);

const beyondSafeInteger = 1n << 100n;
const long = Integer.cast(beyondSafeInteger);
assert.equal(long, beyondSafeInteger);
assert.equal(typeof long, "bigint");

console.log({ number: Number("42"), long });
