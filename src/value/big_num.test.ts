import assert from "node:assert/strict";
import { test } from "node:test";

const { BigNum } = globalThis as typeof globalThis & {
  BigNum: {
    new (format: string, hex: string): {
      readonly format: string;
      readonly byteLength: number;
      toHex(): string;
      toBuffer(): Uint8Array;
    };
    fromHex(format: string, hex: string): {
      readonly format: string;
      readonly byteLength: number;
      toHex(): string;
      toBuffer(): Uint8Array;
    };
    float128(hex: string): { toHex(): string };
  };
};

void test("BigNum preserves a 128 bit payload", () => {
  const bits = "0123456789abcdeffedcba9876543210";
  const value = new BigNum("float128", bits);
  assert.equal(value instanceof BigNum, true);
  assert.equal(Object.getPrototypeOf(value), BigNum.prototype);
  assert.equal(value.format, "float128");
  assert.equal(value.byteLength, 16);
  assert.equal(value.toHex(), bits);
  assert.equal(BigNum.float128(bits).toHex(), bits);
  assert.equal(Buffer.from(value.toBuffer()).toString("hex"), bits);
  assert.equal(Object.prototype.toString.call(value), "[object BigNum]");
  assert.throws(() => Number(value), /explicit numeric conversion/);
  assert.equal(`${value}`, `BigNum.float128(0x${bits})`);
});

void test("format width is enforced", () => {
  assert.throws(() => BigNum.fromHex("float80", "00"), /byte length/);
  assert.throws(() => BigNum.fromHex("float128", "zz"), /hex input/);
});
