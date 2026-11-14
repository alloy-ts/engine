import assert from "node:assert/strict";
import { test } from "node:test";

type NumberWrapper = {
  new (value?: unknown): Number & { readonly value: number; toString(radix?: number): string };
  readonly prototype: object;
  of(value: unknown): Number & { readonly value: number };
  cast(value: unknown): number;
};

const { Float, SmlNum, SmlInt, Integer } = globalThis as typeof globalThis & {
  Float: NumberWrapper;
  SmlNum: NumberWrapper;
  SmlInt: NumberWrapper;
  Integer: {
    new (value?: unknown): BigInt;
    of(value: unknown): BigInt;
    cast(value: unknown): bigint;
  };
};

void test("Float and SmlNum stringify like ECMAScript Number::toString", () => {
  for (const Type of [Float, SmlNum]) {
    assert.equal(new Type(Number.NaN).toString(), "NaN");
    assert.equal(new Type(Infinity).toString(), "Infinity");
    assert.equal(new Type(-Infinity).toString(), "-Infinity");
    assert.equal(new Type(-0).toString(), "0");
    assert.equal(String(new Type(Infinity)), "Infinity");
    assert.equal(`${new Type(-0)}`, "0");
    assert.equal(new Type(5).toString(2), "101");
    assert.equal(new Type(255).toString(16), "ff");
    assert.throws(() => new Type(5).toString(1), RangeError);
    assert.throws(() => new Type(5).toString(37), RangeError);
  }
  assert.equal(new Float(1e21).toString(), "1e+21");
  assert.equal(new Float(1e-7).toString(), "1e-7");
  assert.equal(new Float(0.1).toString(), "0.1");
  assert.equal(new SmlNum(0.1).toString(), String(Math.fround(0.1)));
});

void test("NaN, infinities and negative zero survive the NAPI boundary", () => {
  for (const Type of [Float, SmlNum]) {
    assert.equal(Object.is(new Type(-0).valueOf(), -0), true);
    assert.equal(Object.is(Type.cast(-0), -0), true);
    assert.equal(Object.is(Type.of(-0).value, -0), true);
    assert.equal(Number.isNaN(new Type(Number.NaN).valueOf()), true);
    assert.equal(new Type(Infinity).valueOf(), Infinity);
    assert.equal(new Type(-Infinity).valueOf(), -Infinity);
  }
  // Beyond the reduced-precision range (f32 or binary16) SmlNum overflows to Infinity.
  assert.equal(SmlNum.cast(1e39), Infinity);
  assert.equal(SmlNum.cast(-1e39), -Infinity);
});

void test("bigint input converts exactly like Number(bigint)", () => {
  for (const value of [
    2n ** 53n + 1n,
    2n ** 64n + 2049n,
    248061641579783547148052893441146846n,
    -176066869911571561019881151833792439n,
    752154885170824791502373561466969566n,
  ]) {
    assert.equal(Float.cast(value), Number(value));
  }
  assert.equal(Float.cast(2n ** 1100n), Infinity);
  assert.equal(Float.cast(-(2n ** 1100n)), -Infinity);
  assert.equal(Object.is(Float.cast(0n), 0), true);
});

void test("primitive transfer returns host primitives of the expected type", () => {
  assert.equal(typeof Float.cast(1.5), "number");
  assert.equal(typeof SmlNum.cast(1.5), "number");
  assert.equal(typeof Integer.cast(5), "bigint");
  assert.equal(Integer.cast(5), 5n);
  assert.equal(typeof new Float(1.5), "object");
  assert.equal(typeof new Float(1.5).valueOf(), "number");
  assert.equal(typeof new Integer(5n).valueOf(), "bigint");
});

void test("wrappers keep strict versus loose equality and Number/BigInt mixing rules", () => {
  const float = new Float(1.5);
  assert.equal(float == (1.5 as unknown), true);
  assert.equal(float === (1.5 as unknown), false);
  assert.equal(Object.is(float, 1.5), false);
  assert.notEqual(new Float(1.5), new Float(1.5));
  assert.equal(new Float(Number.NaN) == (Number.NaN as unknown), false);
  assert.equal(new SmlInt(5) == (5n as unknown), true);
  assert.equal(new Integer(5n) == (5 as unknown), true);
  assert.equal((new Float(1.5) as unknown as number) + 1, 2.5);
  assert.throws(() => (new Float(1.5) as unknown as bigint) + 1n, TypeError);
  assert.throws(() => (new Integer(1n) as unknown as number) + (new SmlNum(1) as unknown as number), TypeError);
});

void test("constructors reject unsupported host values, including null", () => {
  for (const Type of [Float, SmlNum, SmlInt] as const) {
    for (const value of [null, "1", true, Symbol("x"), {}, [], () => 1]) {
      assert.throws(() => new Type(value), Error);
      assert.throws(() => Type.of(value), Error);
    }
    assert.equal(new Type().valueOf(), 0);
    assert.equal(new Type(undefined).valueOf(), 0);
    assert.throws(() => (Type as unknown as (v: number) => unknown)(1), TypeError);
  }
  for (const value of [null, "1", true, Symbol("x"), {}, 1.5, Number.NaN, Infinity]) {
    assert.throws(() => new Integer(value), Error);
  }
  assert.equal(new Integer().valueOf(), 0n);
});

void test("borrowed methods reject spoofed receivers", () => {
  for (const Type of [Float, SmlNum, SmlInt]) {
    const proto = Type.prototype as { valueOf(): unknown; toString(): string };
    assert.throws(() => proto.valueOf.call(Object.create(Type.prototype)), /incompatible receiver/);
    assert.throws(() => proto.valueOf.call(new Number(1)), /incompatible receiver/);
    assert.throws(() => proto.toString.call(1), Error);
  }
});
