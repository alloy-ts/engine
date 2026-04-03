import fc from 'fast-check';
import { $ } from '../runtime';

describe('Alloy Shell Production Property Tests', () => {
    // Feature: alloy-shell, Property 1: Injection Safety
    test('interpolated strings never inject commands', async () => {
        await fc.assert(fc.asyncProperty(
            fc.string({ minLength: 0, maxLength: 200 }),
            async (s) => {
                const result = await $`echo ${s}`.nothrow().text();
                expect(result).toBe(s + '\n');
            }
        ), { numRuns: 100 });
    });

    // Feature: alloy-shell, Property 4: Exit Code Propagation
    test('exit codes are propagated correctly', async () => {
        await fc.assert(fc.asyncProperty(
            fc.integer({ min: 0, max: 255 }),
            async (n) => {
                const result = await $`exit ${n}`.nothrow();
                expect(result.exitCode).toBe(n);
            }
        ), { numRuns: 100 });
    });

    // Feature: alloy-shell, Property 6: Pipeline Exit Code
    test('pipeline exit code matches last stage', async () => {
        await fc.assert(fc.asyncProperty(
            fc.integer({ min: 0, max: 255 }),
            fc.integer({ min: 0, max: 255 }),
            async (n1, n2) => {
                const result = await $`exit ${n1} | exit ${n2}`.nothrow();
                expect(result.exitCode).toBe(n2);
            }
        ), { numRuns: 100 });
    });

    // Feature: alloy-shell, Property 20: .throws(false) Equivalence
    test('.throws(false) is equivalent to .nothrow()', async () => {
        await fc.assert(fc.asyncProperty(
            fc.integer({ min: 0, max: 255 }),
            async (n) => {
                const r1 = await $`exit ${n}`.nothrow();
                const r2 = await $`exit ${n}`.throws(false);
                expect(r1.exitCode).toBe(r2.exitCode);
            }
        ), { numRuns: 100 });
    });
});
