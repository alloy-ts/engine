import { test, expect, describe } from "bun:test";
import * as fc from "fast-check";
import * as babel from "@babel/core";
import alloyCpsPlugin from "../../src/babel-plugin-alloy-cps";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const shimPath = join(import.meta.dir, "../../src/core/plugins/async/alloy_async_shim.js");
const shimSource = readFileSync(shimPath, "utf8");

describe("Alloy Async Shim", () => {
  test("Property 7: __alloy_dispatch stores continuation keyed by task_id", () => {
    fc.assert(fc.property(fc.string(), fc.anything(), (op, args) => {
      let capturedTaskId = 0;
      const mockNativeInvoke = (opName: string, argsJson: string) => {
          capturedTaskId++;
          return capturedTaskId;
      };

      const context = {
          nativeInvoke: mockNativeInvoke,
          __alloy_pending: {} as any,
          JSON: JSON,
          __alloy_unhandled_error: (msg: string) => { }
      };

      const shimFn = new Function("ctx", "nativeInvoke", "JSON", "__alloy_unhandled_error", shimSource + "\nctx.__alloy_dispatch = __alloy_dispatch; ctx.__alloy_pending = __alloy_pending;");
      shimFn(context, context.nativeInvoke, context.JSON, context.__alloy_unhandled_error);

      const cont = () => {};
      const taskId = context.__alloy_dispatch(op, args, cont);
      expect(context.__alloy_pending[taskId]).toBe(cont);
    }));
  });

  test("Property 8: __alloy_resolve invokes continuation with (null, result) and removes entry", () => {
      fc.assert(fc.property(fc.integer({min: 1}), fc.anything(), (taskId, result) => {
          let called = false;
          let receivedRes = undefined;
          const cont = (err: any, res: any) => {
              called = true;
              receivedRes = res;
          };

          const pending = { [taskId]: cont } as any;
          const context = {
              __alloy_pending: pending,
              __alloy_unhandled_error: (msg: string) => { },
              JSON: JSON
          };

          const shimFn = new Function("ctx", "JSON", "__alloy_unhandled_error", "pending", shimSource + "\n__alloy_pending = pending; ctx.__alloy_resolve = __alloy_resolve; ctx.__alloy_pending = __alloy_pending;");
          shimFn(context, context.JSON, context.__alloy_unhandled_error, pending);

          context.__alloy_resolve(taskId, result);
          expect(called).toBe(true);
          expect(receivedRes).toEqual(result);
          expect(context.__alloy_pending[taskId]).toBeUndefined();
      }));
  });
});

describe("Babel CPS Plugin", () => {
    async function transform(code: string) {
        const result = await babel.transformAsync(code, {
            plugins: [alloyCpsPlugin],
        });
        return result?.code || "";
    }

    test("Property 3: CPS transform produces valid ES5 with no async syntax", async () => {
        const input = "async function foo() { await bar(); }";
        const output = await transform(input);
        expect(output).not.toContain("async ");
        expect(output).not.toMatch(/\bawait\b/);
        expect(output).toContain("__alloy_dispatch");
    });

    test("Property 4: Every await becomes a __alloy_dispatch call", async () => {
        const input = "async function foo() { await a(); await b(); }";
        const output = await transform(input);
        const matches = output.match(/__alloy_dispatch/g);
        expect(matches?.length).toBe(2);
    });

    test("Property 5: try/catch/await routes errors to catch binding", async () => {
        const input = "async function f() { try { await a(); } catch(e) { handle(e); } }";
        const output = await transform(input);
        expect(output).toContain("var e = __err");
        expect(output).toContain("handle(e)");
    });
});
