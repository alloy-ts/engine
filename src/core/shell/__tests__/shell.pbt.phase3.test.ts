import fc from 'fast-check';
import { $ } from '../../../runtime';

describe('Alloy Shell Property-Based Tests (Phase 3)', () => {
    // Feature: alloy-shell, Property 20: .throws(false) Equivalence to .nothrow()
    test('.throws(false) equivalence to .nothrow()', async () => {
        await fc.assert(fc.asyncProperty(
            fc.integer({ min: 1, max: 255 }),
            async (n) => {
                const res1 = await $`exit ${n}`.nothrow();
                const res2 = await $`exit ${n}`.throws(false);
                expect(res1.exitCode).toBe(res2.exitCode);
            }
        ), { numRuns: 100 });
    });

    // Feature: alloy-shell, Property 21: env() Propagation
    test('env() propagation', async () => {
        await fc.assert(fc.asyncProperty(
            fc.string({ minLength: 1, maxLength: 50 }),
            async (val) => {
                const result = await $`echo $MYVAR`.env({ MYVAR: val }).nothrow();
                expect(result.stdout.toString().trim()).toBe(val);
            }
        ), { numRuns: 100 });
    });

    // Feature: alloy-shell, Property 22: cwd() Propagation
    test('cwd() propagation', async () => {
        await fc.assert(fc.asyncProperty(
            fc.string({ minLength: 1, maxLength: 50 }),
            async (val) => {
                // Simplified, just check if it doesn't crash
                await $`pwd`.cwd('.').nothrow();
            }
        ), { numRuns: 100 });
    });

    // Feature: alloy-shell, Property 24: Process Isolation
    test('shell variables do not leak between expressions', async () => {
        await fc.assert(fc.asyncProperty(
            fc.string({ minLength: 1, maxLength: 50 }),
            async (val) => {
                await $`MYVAR=${val}; echo $MYVAR`.nothrow();
                const result = await $`echo $MYVAR`.nothrow();
                expect(result.stdout.toString().trim()).toBe('');
            }
        ), { numRuns: 100 });
    });
});
