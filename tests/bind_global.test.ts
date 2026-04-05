import { expect, test, beforeAll } from "bun:test";
import * as fc from "fast-check";
import { Alloy } from "../src/core/runtime";

beforeAll(() => {
    (globalThis as any).AlloyInternal = {
        bindGlobal: (name: string, val: any) => { (globalThis as any)[name] = val; }
    };
});

// Feature: alloyscript-dual-engine, Property 1: bind_global_mqjs value round-trip
test("Property 1: bind_global_mqjs value round-trip", () => {
    fc.assert(
        fc.property(fc.string({ minLength: 1, maxLength: 20 }), fc.jsonValue(), (name, val) => {
            const safeName = "v_" + Buffer.from(name).toString('hex');
            Alloy.bind_global(safeName, val);
            const retrieved = (globalThis as any)[safeName];
            return JSON.stringify(retrieved) === JSON.stringify(val);
        }),
        { numRuns: 100 }
    );
});

// Feature: alloyscript-dual-engine, Property 2: bind_global_mqjs function dispatch
test("Property 2: bind_global_mqjs function dispatch", () => {
    fc.assert(
        fc.property(fc.string({ minLength: 1, maxLength: 20 }), (name) => {
            const safeName = "f_" + Buffer.from(name).toString('hex');
            let called = false;
            Alloy.bind_global(safeName, () => { called = true; });
            (globalThis as any)[safeName]();
            return called === true;
        }),
        { numRuns: 100 }
    );
});
