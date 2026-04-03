import { expect, test, describe } from "bun:test";
import * as fc from "fast-check";

describe("IPC Crypto Properties", () => {
  test("Property 10: AlloyMessage serialisation round-trip", () => {
    // Feature: alloyscript-dual-engine, Property 10: AlloyMessage serialisation round-trip
    fc.assert(
      fc.property(
        fc.record({
          seq: fc.bigInt({ min: 0n }),
          type: fc.integer({ min: 1, max: 3 }),
          payload: fc.string()
        }),
        (msg) => {
          const serialized = JSON.stringify(msg, (key, value) =>
            typeof value === 'bigint' ? value.toString() : value
          );
          const deserialized = JSON.parse(serialized);
          expect(deserialized.seq).toBe(msg.seq.toString());
          expect(deserialized.type).toBe(msg.type);
          expect(deserialized.payload).toBe(msg.payload);
        }
      ),
      { numRuns: 100 }
    );
  });
});
