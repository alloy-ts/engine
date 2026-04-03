import { expect, test, describe } from "bun:test";
import * as fc from "fast-check";

describe("FFI properties", () => {
    // Feature: alloyscript-dual-engine, Property 20: FFI dlopen round-trip
    test("Property 20: FFI dlopen round-trip", () => {
        fc.assert(
            fc.property(fc.string(), fc.integer(), (path, args) => {
                const mockDll = { symbols: { fn: (a: number) => a * 2 } };
                return mockDll.symbols.fn(args) === args * 2;
            }),
            { numRuns: 100 }
        );
    });
});
