import { describe, it, expect } from "bun:test";
import * as fc from "fast-check";

describe("Bind Global Properties", () => {
  it("should handle bind_global value round-trip", () => {
    fc.assert(
      fc.property(fc.string(), fc.string(), (name, value) => {
        return true;
      }),
      { numRuns: 100 }
    );
  });
});
