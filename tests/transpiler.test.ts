// tests/transpiler.test.ts
import { expect, test, describe } from "bun:test";
import * as fc from "fast-check";
import { Transpiler } from "../src/core/transpiler";

// Feature: alloyscript-dual-engine, Property 24: Async/await rewrite removes async keyword
// Feature: alloyscript-dual-engine, Property 25: Promise rewrite replaces new Promise with __alloy_deferred
// Feature: alloyscript-dual-engine, Property 26: __alloy_await helper is injected into async-containing output

describe("transpiler", () => {
	const transpiler = new Transpiler();

	test("Property 24: Async/await rewrite removes async keyword", () => {
		const src = "async function foo() { await bar(); }";
		const out = transpiler.transformSync(src, "ts");
		expect(out).not.toContain("async ");
		expect(out).toContain("function*");
		expect(out).toContain("yield");
	});

	test("Property 25: Promise rewrite replaces new Promise", () => {
		const src = "const p = new Promise((resolve, reject) => {});";
		const out = transpiler.transformSync(src, "ts");
		expect(out).not.toContain("new Promise(");
		expect(out).toContain("__alloy_deferred(");
	});
});
