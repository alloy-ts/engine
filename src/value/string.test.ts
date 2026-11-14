import assert from "node:assert/strict";
import { test } from "node:test";

void test("the global String remains available after the addon init hook", () => {
  assert.equal(String, globalThis.String);
  assert.equal(typeof globalThis.Value, "function");
  assert.equal(typeof globalThis.Integer, "function");
});

void test("global String round-trips UTF-8 text", () => {
  const text = String.fromCodePoint(0x41, 0x00e9, 0x1f980);
  const encoded = new TextEncoder().encode(text);
  const buffer = Buffer.from(text, "utf8");

  assert.deepEqual(buffer, Buffer.from(encoded));
  assert.equal(new TextDecoder("utf-8", { fatal: true }).decode(encoded), text);
  assert.equal(buffer.toString("utf8"), text);
});

void test("global String round-trips UTF-16 little-endian code units", () => {
  const text = String.fromCodePoint(0x61, 0x00e9, 0x1f980);
  const encoded = Buffer.from(text, "utf16le");

  assert.equal(encoded.byteLength, text.length * 2);
  assert.equal(encoded.toString("utf16le"), text);
  assert.equal(encoded.readUInt16LE(0), 0x61);
  assert.equal(encoded.readUInt16LE(2), 0x00e9);
  assert.equal(encoded.readUInt16LE(4), 0xd83e);
  assert.equal(encoded.readUInt16LE(6), 0xdd80);
});

void test("String wrappers retain native UTF-16 semantics", () => {
  const text = String.fromCodePoint(0x1f980);
  const boxed = new String(text);

  assert.equal(Object.getPrototypeOf(boxed), String.prototype);
  assert.equal(boxed.valueOf(), text);
  assert.equal(boxed.length, 2);
  assert.equal(boxed.charCodeAt(0), 0xd83e);
  assert.equal(boxed.charCodeAt(1), 0xdd80);
});

void test("UTF-16 preserves lone surrogates while UTF-8 encodes replacement", () => {
  const loneSurrogate = String.fromCharCode(0xd800);
  const utf16 = Buffer.from(loneSurrogate, "utf16le");
  const utf8 = Buffer.from(loneSurrogate, "utf8");

  assert.equal(utf16.toString("utf16le"), loneSurrogate);
  assert.deepEqual([...utf8], [0xef, 0xbf, 0xbd]);
  assert.equal(utf8.toString("utf8"), "\ufffd");
});
