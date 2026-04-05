import { expect, test, beforeAll } from "bun:test";
import { build } from "../src/core/build";
import { Transpiler } from "../src/core/transpiler";
import { readFile, writeFile, mkdir, rm } from "node:fs/promises";
import { join } from "node:path";

const TEST_DIR = "tests/build_test_tmp";

beforeAll(async () => {
    await rm(TEST_DIR, { recursive: true, force: true });
    await mkdir(TEST_DIR, { recursive: true });
});

test("Alloy.build entry-point coverage", async () => {
    const entry = join(TEST_DIR, "main.ts");
    await writeFile(entry, 'import { foo } from "./utils"; console.log(foo);');
    await writeFile(join(TEST_DIR, "utils.ts"), 'export const foo = 42;');

    const result = await build({ entrypoints: [entry], outdir: join(TEST_DIR, "dist") });
    expect(result.success).toBe(true);
    expect(result.outputs.length).toBeGreaterThanOrEqual(2);
    expect(result.outputs.some(o => o.path.includes("main.js"))).toBe(true);
    expect(result.outputs.some(o => o.path.includes("utils.js"))).toBe(true);
});

test("Alloy.build minification reduces size", async () => {
    const entry = join(TEST_DIR, "mini.ts");
    await writeFile(entry, 'function test() {    const x = 1;    return x;    }');

    const unminified = await build({ entrypoints: [entry], minify: false });
    const minified = await build({ entrypoints: [entry], minify: true });

    expect(minified.outputs[0].size).toBeLessThan(unminified.outputs[0].size);
});
