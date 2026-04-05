import { expect, test } from "bun:test";
import * as fc from "fast-check";
import { build } from "../src/core/build";

test("Alloy.build entry-point coverage", async () => {
    /* Feature: alloyscript-dual-engine, Property 27: Alloy.build entry-point coverage */
    const result = await build({ entrypoints: ["src/main.ts"] });
    expect(result.success).toBe(true);
    expect(result.outputs.length).toBeGreaterThan(0);
});

test("Alloy.build minified output is no larger than unminified", async () => {
    /* Feature: alloyscript-dual-engine, Property 28: Alloy.build minified output size */
    const unminified = await build({ entrypoints: ["src/main.ts"], minify: false });
    const minified = await build({ entrypoints: ["src/main.ts"], minify: true });
    expect(minified.outputs[0].size).toBeLessThanOrEqual(unminified.outputs[0].size);
});
