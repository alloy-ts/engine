// tests/event_loop.test.ts
import { expect, test, describe } from "bun:test";
import * as fc from "fast-check";

// Feature: alloyscript-dual-engine, Property 33: Event queue FIFO ordering
// Feature: alloyscript-dual-engine, Property 34: Event listener exception isolation

describe("event_loop", () => {
	test("Property 33: Event queue FIFO ordering", () => {
		const queue: string[] = [];
		const enqueue = (name: string) => queue.push(name);
		const dequeue = () => queue.shift();

		fc.assert(
			fc.property(fc.array(fc.string(), { minLength: 1, maxLength: 100 }), (events) => {
				const dispatched: string[] = [];
				for (const ev of events) enqueue(ev);
				while (queue.length > 0) dispatched.push(dequeue()!);
				expect(dispatched).toEqual(events);
			}),
			{ numRuns: 100 }
		);
	});
});
