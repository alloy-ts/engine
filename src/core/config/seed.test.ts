import { test, expect } from "bun:test";
import { randomizeOrder } from "./seed";

test("Property 9: Seed Determinism", () => {
	// Feature: alloyconfig-toml, Property 9: Seed Determinism
	const files = ["a", "b", "c", "d", "e"];
	const seed = 12345;

	const order1 = randomizeOrder(files, seed);
	const order2 = randomizeOrder(files, seed);

	expect(order1).toEqual(order2);
	expect(order1).not.toEqual(files); // Check if actually randomized
});
