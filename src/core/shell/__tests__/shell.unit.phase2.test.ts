import { expect, test, describe } from "bun:test";
import { $ } from "../../runtime";

describe("Alloy Shell Unit Tests (Phase 2)", () => {
    test("concurrency with Promise.all", async () => {
        const [r1, r2] = await Promise.all([
            $`echo 1`.nothrow(),
            $`echo 2`.nothrow()
        ]);
        expect(r1.exitCode).toBe(0);
        expect(r2.exitCode).toBe(0);
    });

    test("env propagation", async () => {
        const res = await $`echo $FOO`.env({ FOO: 'bar' }).nothrow();
        expect(res.exitCode).toBe(0);
    });
});
