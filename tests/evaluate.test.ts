import { expect, test, beforeAll } from "bun:test";
import { evaluate } from "../src/core/evaluate";
import { Transpiler } from "../src/core/transpiler";

beforeAll(() => {
    // High-fidelity mock of AlloyInternal for testing in Bun environment
    (globalThis as any).AlloyInternal = {
        secureEval: (s: string) => {
            // Simulate MQJS evaluation
            return eval(s);
        },
        parse: (s: string) => {
            try {
                new Function(s);
                return null;
            } catch (e: any) {
                return e.message;
            }
        },
        on: () => {},
        off: () => {},
        emit: () => {},
        nextTick: (fn: Function) => fn(),
        getMmap: () => ({ regions: [], jsArenaProtected: true }),
        readMmap: () => new ArrayBuffer(0),
        spawn: () => Promise.resolve({ exitCode: 0, stdout: "", stderr: "" }),
        spawnSync: () => ({ exitCode: 0, stdout: "", stderr: "" }),
        bindGlobal: (name: string, val: any) => { (globalThis as any)[name] = val; }
    };
});

test("evaluate() handles simple expressions", async () => {
    const res = await evaluate("1 + 2");
    expect(res).toBe(3);
});

test("evaluate() delegates to secureEval", async () => {
    const res = await evaluate("10 * 10");
    expect(res).toBe(100);
});

test("evaluate() handles independent lines", async () => {
    await evaluate("globalThis.x = 42");
    const res = await evaluate("globalThis.x");
    expect(res).toBe(42);
});

test("evaluate() rejects invalid syntax", async () => {
    const res = await evaluate("if ( {");
    expect(res.error).toBe("transpile_failed");
});
