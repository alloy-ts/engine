import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

// The test runner preloads the addon, so observe a fresh process that
// snapshots the host intrinsics before and after loading it.
const addon = fileURLToPath(new URL("../../dist/index.js", import.meta.url));

function runIsolated(source: string): unknown {
  const output = execFileSync(process.execPath, ["--input-type=module", "-e", source], {
    env: { ...process.env, ADDON: addon },
    encoding: "utf8",
  });
  return JSON.parse(output);
}

void test("loading the addon leaves host built-ins and their prototypes unchanged", () => {
  const result = runIsolated(`
    const names = ["Number", "BigInt", "Boolean", "String", "Object", "Function", "Reflect", "Symbol", "Buffer"];
    const describe = (target) => target == null ? null : Reflect.ownKeys(target).map((key) => {
      const d = Reflect.getOwnPropertyDescriptor(target, key);
      return [String(key), d.value, d.get, d.set, d.writable, d.enumerable, d.configurable];
    });
    const snapshot = () => names.map((name) => {
      const value = globalThis[name];
      return { name, value, own: describe(value), proto: describe(value.prototype),
        parent: value.prototype && Object.getPrototypeOf(value.prototype) };
    });
    const sameDescriptors = (a, b) => a === b || (a.length === b.length &&
      a.every((entry, i) => entry.every((part, j) => Object.is(part, b[i][j]))));
    const before = snapshot();
    const globalsBefore = new Set(Reflect.ownKeys(globalThis).map(String));
    await import(process.env.ADDON);
    const after = snapshot();
    const changed = before.filter((b, i) => {
      const a = after[i];
      return a.value !== b.value || a.parent !== b.parent ||
        !sameDescriptors(a.own, b.own) || !sameDescriptors(a.proto, b.proto);
    }).map((b) => b.name);
    const added = Reflect.ownKeys(globalThis).map(String).filter((k) => !globalsBefore.has(k)).sort();
    console.log(JSON.stringify({ changed, added, primitive: [typeof 1, typeof 1n, (5).toString(), String(2n ** 64n)] }));
  `);
  assert.deepEqual(result, {
    changed: [],
    added: ["BigNum", "CoreObject", "Float", "Integer", "SmlFlt", "SmlInt", "SmlNum", "Value"],
    primitive: ["number", "bigint", "5", "18446744073709551616"],
  });
});

void test("wrapper construction uses intrinsics captured at load, not patched globals", () => {
  const result = runIsolated(`
    await import(process.env.ADDON);
    Reflect.getPrototypeOf = () => { throw new Error("patched getPrototypeOf"); };
    Reflect.setPrototypeOf = () => { throw new Error("patched setPrototypeOf"); };
    Object.create = () => { throw new Error("patched Object.create"); };
    const out = [
      new Float(1.5).valueOf(), Float.of(2.5).valueOf(),
      new SmlNum(1.5).valueOf(), SmlNum.of(0.5).valueOf(),
      new SmlInt(3).valueOf(), SmlInt.of(4).valueOf(),
      String(new Integer(5n).valueOf()), String(Integer.of(6n).valueOf()),
      Object.getPrototypeOf.call(null, CoreObject.create(null)) === null,
    ];
    console.log(JSON.stringify(out));
  `);
  assert.deepEqual(result, [1.5, 2.5, 1.5, 0.5, 3, 4, "5", "6", true]);
});
