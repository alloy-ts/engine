import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const bin = fileURLToPath(new URL("../bin/alloy-engine.mjs", import.meta.url));

function run(args: string[], input?: string) {
  return spawnSync(process.execPath, [bin, ...args], {
    encoding: "utf8",
    input,
  });
}

void test("eval 5-888 prints -883", () => {
  const result = run(["eval", "5-888"]);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, "-883\n");
  assert.equal(result.stderr, "");
});

void test("eval - reads the source from stdin", () => {
  const result = run(["eval", "-"], "1 + 2 * 3");
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, "7\n");
});

void test("a syntax error exits non-zero and prints the error", () => {
  const result = run(["eval", "1 +"]);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /SyntaxError/);
  assert.equal(result.stdout, "");
});

void test("a runtime error exits non-zero and prints the error", () => {
  const result = run(["eval", "missing"]);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /ReferenceError/);
  assert.equal(result.stdout, "");
});

void test("--help prints usage and exits 0", () => {
  const result = run(["--help"]);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /Usage: alloy-engine eval/);
});
