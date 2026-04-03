// src/core/config/__tests__/property_test.ts
import { expect, test, describe } from "bun:test";
import * as fc from "fast-check";
import { PrettyPrinter } from "../pretty_printer";
import { ConfigLoader } from "../loader";
import { parse } from "smol-toml";

describe("Property Tests", () => {
  const printer = new PrettyPrinter();
  const loader = new ConfigLoader();

  const arbitraryConfig = fc.record({
    target: fc.constantFrom("local-first", "server-first"),
    logLevel: fc.constantFrom("debug", "warn", "error"),
    telemetry: fc.boolean(),
    define: fc.dictionary(fc.string({alphabet: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_"}), fc.string()),
    test: fc.record({
      root: fc.string(),
      coverage: fc.boolean(),
      randomize: fc.boolean(),
      seed: fc.integer(),
    }, { requiredKeys: ["root", "coverage", "randomize"] })
  });

  test("Property 1: parse(prettyPrint(config)) deep-equals config", () => {
    fc.assert(fc.property(arbitraryConfig, (config: any) => {
      const toml = printer.print(config);
      const parsed = parse(toml);
      expect(parsed).toEqual(config);
    }), { numRuns: 100 });
  });

  test("Property 3: env var interpolation works", () => {
    fc.assert(fc.property(fc.constantFrom("A", "B", "C"), fc.string({ alphabet: "abcdefghijklmnopqrstuvwxyz0123456789" }), (suffix: string, varValue: string) => {
      const varName = "TEST_" + suffix;
      process.env[varName] = varValue;
      const raw = { jsxImportSource: `$${varName}` };
      const interpolated = loader.interpolate(raw);
      expect((interpolated as any).jsxImportSource).toBe(varValue);
      delete process.env[varName];
    }), { numRuns: 100 });
  });

  test("Property 3 bis: env var interpolation with curly braces works", () => {
    fc.assert(fc.property(fc.constantFrom("A", "B", "C"), fc.string({ alphabet: "abcdefghijklmnopqrstuvwxyz0123456789" }), (suffix: string, varValue: string) => {
      const varName = "TEST_CURLY_" + suffix;
      process.env[varName] = varValue;
      const raw = { jsxImportSource: `\${${varName}}` };
      const interpolated = loader.interpolate(raw);
      expect((interpolated as any).jsxImportSource).toBe(varValue);
      delete process.env[varName];
    }), { numRuns: 100 });
  });
});
