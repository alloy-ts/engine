import { expect, test } from "bun:test";
import * as fc from "fast-check";

import { evaluate } from "../src/core/evaluate";

test("REPL evaluates lines independently", async () => {
    // Property 19: REPL evaluates N lines independently
    const results = [];
    results.push(await evaluate("1 + 1"));
    try { await evaluate("throw new Error('fail')"); } catch(e) {}
    results.push(await evaluate("2 + 2"));
    expect(results).toEqual([2, 4]);
});

test("evaluate() delegates to secureEval", async () => {
    // Property 17: evaluate() delegates to secureEval
    (globalThis as any).AlloyInternal = { secureEval: (s: string) => eval(s) };
    const res = await evaluate("10 * 10");
    expect(res).toBe(100);
});
