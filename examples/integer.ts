import assert from "node:assert/strict";

// Run after building: node --import ./dist/index.js examples/integer.ts

type IntegerValue = number | bigint;

interface IntegerInstance {
  readonly value: bigint;
  readonly bits: number | undefined;
  readonly unsigned: boolean;
  toString(radix?: number): string;
  valueOf(): bigint;
}

interface IntegerConstructor {
  new (
    value?: IntegerInstance | IntegerValue,
    bits?: number,
    unsigned?: boolean,
  ): IntegerInstance;
  readonly prototype: IntegerInstance;
  of(
    value: IntegerInstance | IntegerValue,
    bits?: number,
    unsigned?: boolean,
  ): IntegerInstance;
  cast(
    value: IntegerInstance | IntegerValue,
    bits?: number,
    unsigned?: boolean,
  ): bigint;
  asIntN(bits: number, value: IntegerInstance | IntegerValue): bigint;
  asUintN(bits: number, value: IntegerInstance | IntegerValue): bigint;
}

const { Integer } = globalThis as typeof globalThis & {
  Integer: IntegerConstructor;
};

assert.equal(typeof Integer, "function");
assert.equal(Integer instanceof Function, true);

const answer = new Integer(42, 32);
assert.equal(Object.getPrototypeOf(answer), Integer.prototype);
assert.equal(answer instanceof Integer, true);
assert.equal(answer.value, 42n);
assert.equal(answer.bits, 32);

// A BigInt-backed Integer has no inherent width and can exceed 128 bits.
const huge = (1n << 4096n) + 7n;
const long = new Integer(huge);
assert.equal(long.valueOf(), huge);
assert.equal(long.toString(), huge.toString());

// `of` creates a wrapped object; `cast` returns a JS primitive.
assert.equal(Integer.of(255, 8, true).value, 255n);
assert.equal(Integer.cast(255, 8, true), 255n);

console.log({ answer: answer.value, long: long.value });
