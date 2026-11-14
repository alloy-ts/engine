import assert from "node:assert/strict";
import { test } from "node:test";

const runtimeGlobal = (
  globalThis as typeof globalThis & {
    Integer: {
      new (value?: number | bigint, bits?: number, unsigned?: boolean): any;
      readonly prototype: object;
      readonly new: {
        new (value?: number | bigint, bits?: number, unsigned?: boolean): any;
        (value?: number | bigint, bits?: number, unsigned?: boolean): any;
      };
      of(value: number | bigint, bits?: number, unsigned?: boolean): any;
      cast(value: number | bigint, bits?: number, unsigned?: boolean): number | bigint;
      readonly SML_BITS: number;
      readonly NORMAL_CELL_BITS: number;
      asUintN(bits: number, value: number | bigint): bigint;
    };
    SmlInt: {
      new (value?: number | bigint, bits?: number, unsigned?: boolean): any;
      readonly prototype: object;
      readonly new: { new (value?: number | bigint, bits?: number): any; (value?: number | bigint, bits?: number): any };
      of(value: number | bigint, bits?: number, unsigned?: boolean): any;
      cast(value: number | bigint, bits?: number, unsigned?: boolean): number | bigint;
    };
    Value: {
      readonly prototype: object;
      [Symbol.hasInstance](value: unknown): boolean;
    };
  }
);
const { Integer: BigInteger, SmlInt: Integer, Value } = runtimeGlobal;

void test("module initialization registers the SmlInt subtype", () => {
  assert.equal(runtimeGlobal.SmlInt, Integer);
  assert.equal(runtimeGlobal.Integer, BigInteger);
  assert.equal(typeof runtimeGlobal.Integer, "function");
  assert.equal(typeof runtimeGlobal.Value, "function");
  assert.notEqual(runtimeGlobal.SmlInt, runtimeGlobal.Integer);
});

void test("new Integer creates a wrapped Integer.prototype instance", () => {
  const value = new Integer(42, 32);

  assert.equal(typeof Integer, "function");
  assert.equal(Integer instanceof Function, true);
  assert.equal(typeof Integer.prototype, "object");
  assert.equal(Object.getPrototypeOf(value), Integer.prototype);
  assert.equal(value instanceof Integer, true);
  assert.equal(value.constructor, Integer);
  assert.equal(value.value, 42);
  assert.equal(value.bits, 32);
  assert.equal(value.bytes, 4);
  assert.equal(value.unsigned, false);
  assert.deepEqual(Reflect.ownKeys(value), []);
  assert.equal(value instanceof Value, true);
  assert.equal(Object.getPrototypeOf(Integer), Value);
  assert.equal(Object.getPrototypeOf(Integer.prototype), Number.prototype);
  assert.equal(Object.getPrototypeOf(runtimeGlobal.SmlInt.prototype), Number.prototype);
});

void test("new SmlInt defaults to the target small width", () => {
  const value = new Integer();

  assert.equal(value.value, 0);
  assert.equal(typeof value.valueOf(), "number");
  assert.equal(value.bits, Integer.SML_BITS);
  assert.equal(value.unsigned, false);
});

void test("SmlInt enforces the target small-integer width", () => {
  const min = -(2 ** (Integer.SML_BITS - 1));
  const max = 2 ** (Integer.SML_BITS - 1) - 1;
  assert.equal(new Integer(min).value, min);
  assert.equal(new Integer(max).value, max);
  assert.throws(() => new Integer(min - 1), /target .*bit range/);
  assert.throws(() => Integer.of(max + 1), /target .*bit range/);
  assert.equal(new Integer(2 ** Integer.SML_BITS - 1, Integer.SML_BITS, true).value, 2 ** Integer.SML_BITS - 1);
  assert.equal(typeof Integer.cast(42), "number");
});

void test("Integer requires new while cast remains unboxed", () => {
  assert.throws(
    () => Reflect.apply(Integer, undefined, [42, 32]),
    /requires 'new'/,
  );
  assert.equal(Integer.cast(42, 32), 42);
});

void test("Integer.of creates the same prototype-linked representation", () => {
  const value = Integer.of(255, 8, true);

  assert.equal(Object.getPrototypeOf(value), Integer.prototype);
  assert.equal(value instanceof Integer, true);
  assert.equal(value.value, 255);
  assert.equal(value.unsigned, true);
});

void test("Integer.new is the constructor alias", () => {
  assert.equal(Integer.new, Integer);

  const called = Integer.new(9, 8);
  const constructed = new Integer.new(10, 8);

  assert.equal(called instanceof Integer, true);
  assert.equal(constructed instanceof Integer, true);
  assert.equal(called.value, 9);
  assert.equal(constructed.value, 10);
});

void test("slot-backed instanceof rejects prototype spoofing", () => {
  assert.equal(Object.create(Integer.prototype) instanceof Integer, false);
  assert.equal(Object.create(Integer.prototype) instanceof Value, false);
});

void test("Value coercion is composed onto numeric prototypes", () => {
  const value = new Integer(10n);

  assert.equal(Object.hasOwn(Value.prototype, Symbol.toPrimitive), true);
  // Numeric wrappers inherit Number.prototype, so Value's shared coercion
  // behavior is composed as an own method instead of adding another ancestor.
  assert.equal(Object.hasOwn(Integer.prototype, Symbol.toPrimitive), true);
  assert.equal(value + 5, 15);
  assert.throws(() => value + 5n, TypeError);
  assert.equal(`${value}`, "10");
  assert.equal(BigInt(value), 10n);
  assert.equal(Object.prototype.toString.call(value), "[object SmlInt]");
});

void test("Integer uses BigInt for every width and spills beyond 128 bits", () => {
  const huge = (1n << 4096n) + 7n;
  const value = new BigInteger(huge);
  assert.equal(value.value, huge);
  assert.equal(value.valueOf(), huge);
  assert.equal(value instanceof BigInteger, true);
  assert.equal(value instanceof Value, true);
  assert.equal(Object.getPrototypeOf(BigInteger.prototype), BigInt.prototype);
  assert.equal(value.toString(16), huge.toString(16));
  assert.equal(Object.prototype.toString.call(value), "[object Integer]");
  assert.equal(Object.create(BigInteger.prototype) instanceof BigInteger, false);
  assert.equal(BigInteger.cast(255, 8, false), -1n);
  assert.equal(BigInteger.asUintN(16, -1n), 65535n);
});

void test("new.target preserves a derived prototype", () => {
  class ExtendedInteger extends Integer {}

  const value = new ExtendedInteger(7, 8);

  assert.equal(Object.getPrototypeOf(value), ExtendedInteger.prototype);
  assert.equal(value instanceof ExtendedInteger, true);
  assert.equal(value instanceof Integer, true);
  assert.equal(value.value, 7);
});
