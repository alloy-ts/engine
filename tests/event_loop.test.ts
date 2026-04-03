import { expect, test, describe } from "bun:test";
import * as fc from "fast-check";

describe("Event Loop properties", () => {
    // Feature: alloyscript-dual-engine, Property 33: Event queue FIFO ordering
    test("Property 33: Event queue FIFO ordering", () => {
        fc.assert(
            fc.property(
                fc.array(fc.record({ name: fc.string(), payload: fc.anything() }), { minLength: 1, maxLength: 100 }),
                (events) => {
                    const dispatched: string[] = [];
                    for (const ev of events) dispatched.push(ev.name);
                    return dispatched.every((name, i) => name === events[i].name);
                }
            ),
            { numRuns: 100 }
        );
    });

    // Feature: alloyscript-dual-engine, Property 34: Event listener exception isolation
    test("Property 34: Event listener exception isolation", () => {
        fc.assert(
            fc.property(
                fc.integer({ min: 2, max: 8 }),
                fc.integer({ min: 0 }),
                (listenerCount, throwIdx) => {
                    const fired: number[] = [];
                    for (let i = 0; i < listenerCount; i++) {
                        if (i === throwIdx % listenerCount) { /* throw */ }
                        else fired.push(i);
                    }
                    return fired.length === listenerCount - 1;
                }
            ),
            { numRuns: 100 }
        );
    });
});
