import { expect, test, describe } from "bun:test";
import { $ } from "../../runtime";

describe("Alloy Shell Unit Tests (Phase 1)", () => {
    test("echo builtin", async () => {
        const res = await $`echo hello world`.nothrow();
        expect(res.exitCode).toBe(0);
    });

    test("true and false builtins", async () => {
        expect((await $`true`.nothrow()).exitCode).toBe(0);
        expect((await $`false`.nothrow()).exitCode).toBe(1);
    });
});
