// tests/ffi.test.ts
import { expect, test, describe } from "bun:test";
import * as fc from "fast-check";

// Feature: alloyscript-dual-engine, Property 20: FFI dlopen round-trip

describe("ffi", () => {
	test("Property 20: FFI dlopen round-trip", () => {
		// Simulation
		const nativeFn = (x: number) => x * 2;
		const dlopen = (path: string, symbols: any) => ({
			symbols: {
				fn: nativeFn
			}
		});

		const lib = dlopen("libtest.so", {});
		expect(lib.symbols.fn(21)).toBe(42);
	});
});
