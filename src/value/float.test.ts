import assert from "node:assert/strict";
import { test } from "node:test";

const runtimeGlobal = globalThis as typeof globalThis & {
  Value: { readonly prototype: object };
  Float: {
    new (value?: number | bigint): any;
    readonly prototype: object;
    of(value: number | bigint): any;
    cast(value: number | bigint): number;
    isNaN(value: number | bigint): boolean;
    isFinite(value: number | bigint): boolean;
    readonly NORMAL_CELL_BITS: number;
  };
  SmlFlt: {
    new (value?: number | bigint): any;
    readonly prototype: object;
    readonly BITS: number;
    of(value: number | bigint): any;
    cast(value: number | bigint): number;
  };
  SmlNum: {
    new (value?: number | bigint): any;
    readonly prototype: object;
    readonly BITS: number;
    of(value: number | bigint): any;
    cast(value: number | bigint): number;
  };
};

const { Float, SmlFlt, SmlNum, Value } = runtimeGlobal;

void test("Float is a separate global primitive wrapper", () => {
  const value = new Float(2.5);

  assert.equal(typeof Float, "function");
  assert.equal(Object.getPrototypeOf(value), Float.prototype);
  assert.equal(runtimeGlobal.SmlFlt, runtimeGlobal.SmlNum);
  assert.notEqual(runtimeGlobal.SmlNum, runtimeGlobal.Float);
  assert.equal(SmlNum, SmlFlt);
  assert.equal(Object.getPrototypeOf(SmlFlt.prototype), Number.prototype);
  assert.equal(value instanceof Float, true);
  assert.equal(value instanceof Value, true);
  assert.equal(value.value, 2.5);
  assert.equal(value.valueOf(), 2.5);
  assert.equal(value.toString(), "2.5");
  assert.equal(Object.prototype.toString.call(value), "[object Float]");
  assert.equal(Float.NORMAL_CELL_BITS === 32 || Float.NORMAL_CELL_BITS === 64, true);
});

void test("SmlNum uses target reduced precision while remaining a Number", () => {
  const value = new SmlNum(1.5);
  const rounded = SmlNum.BITS === 32 ? Math.fround(1.1) : 1.099609375;

  assert.equal(SmlNum.BITS === 16 || SmlNum.BITS === 32, true);
  assert.equal(value.value, 1.5);
  assert.equal(value.valueOf(), 1.5);
  assert.equal(value instanceof SmlNum, true);
  assert.equal(value instanceof Value, true);
  assert.equal(Object.getPrototypeOf(SmlNum.prototype), Number.prototype);
  assert.equal(Object.prototype.toString.call(value), "[object SmlNum]");
  assert.equal(SmlNum.cast(1.5), 1.5);
  assert.equal(SmlNum.cast(1.1), rounded);
  assert.equal(Float.cast(1.1), 1.1);
});

void test("Float preserves Number compatibility without replacing Number", () => {
  assert.equal(Number(2.5), 2.5);
  assert.equal(Float.cast(2.5), 2.5);
  assert.equal(Float.of(2.5).value, 2.5);
  assert.equal(Float.isNaN(Number.NaN), true);
  assert.equal(Float.isFinite(2.5), true);
});

void test("Float supports bigint conversion", () => {
  const value = new Float(3n);

  assert.equal(value.value, 3);
  assert.equal(value instanceof Float, true);
});
