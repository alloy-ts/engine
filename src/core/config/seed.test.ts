// src/core/config/seed.test.ts
import { expect, test, describe } from "bun:test";
import { randomizeOrder } from "./seed";

describe("Seed Determinism", () => {
	test("Property 9: same seed produces same test order", () => {
		const files = ["test1.ts", "test2.ts", "test3.ts", "test4.ts", "test5.ts"];
		const seed = 42;

		const order1 = randomizeOrder(files, seed);
		const order2 = randomizeOrder(files, seed);
		const order3 = randomizeOrder(files, seed + 1);

		expect(order1).toEqual(order2);
		expect(order1).not.toEqual(order3);
	});
});
