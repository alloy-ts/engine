import fc from 'fast-check';
import { $ } from '../../../runtime';

describe('Alloy Shell Property-Based Tests (Phase 1)', () => {
    // Feature: alloy-shell, Property 1: Injection Safety
    test('interpolated strings never inject commands', async () => {
        await fc.assert(fc.asyncProperty(
            fc.string({ minLength: 0, maxLength: 200 }),
            async (s) => {
                const result = await $`echo ${s}`.nothrow();
                expect(result.stdout.toString()).toBe(s + '\n');
                expect(result.exitCode).toBe(0);
            }
        ), { numRuns: 100 });
    });

    // Feature: alloy-shell, Property 2: Escape Idempotence
    test('$.escape is idempotent', () => {
        fc.assert(fc.property(
            fc.string(),
            (s) => {
                expect($.escape($.escape(s))).toBe($.escape(s));
            }
        ), { numRuns: 100 });
    });

    // Feature: alloy-shell, Property 3: Escape Round-Trip
    test('escape round-trip', async () => {
        await fc.assert(fc.asyncProperty(
            fc.string(),
            async (s) => {
                const result1 = await $`echo ${s}`.nothrow();
                const result2 = await $`echo ${{ raw: $.escape(s) }}`.nothrow();
                expect(result1.stdout.toString()).toBe(result2.stdout.toString());
            }
        ), { numRuns: 100 });
    });

    // Feature: alloy-shell, Property 4: Exit Code Propagation — nothrow
    test('exit code propagation (nothrow)', async () => {
        await fc.assert(fc.asyncProperty(
            fc.integer({ min: 0, max: 255 }),
            async (n) => {
                const result = await $`exit ${n}`.nothrow();
                expect(result.exitCode).toBe(n);
            }
        ), { numRuns: 100 });
    });

    // Feature: alloy-shell, Property 5: Exit Code Propagation — throw
    test('exit code propagation (throw)', async () => {
        await fc.assert(fc.asyncProperty(
            fc.integer({ min: 1, max: 255 }),
            async (n) => {
                try {
                    await $`exit ${n}`;
                    fail('Should have thrown');
                } catch (e: any) {
                    expect(e.exitCode).toBe(n);
                }
            }
        ), { numRuns: 100 });
    });

    // Feature: alloy-shell, Property 6: Pipeline Exit Code
    test('pipeline exit code is last stage', async () => {
        await fc.assert(fc.asyncProperty(
            fc.integer({ min: 0, max: 255 }),
            fc.integer({ min: 0, max: 255 }),
            async (n1, n2) => {
                const result = await $`exit ${n1} | exit ${n2}`.nothrow();
                expect(result.exitCode).toBe(n2);
            }
        ), { numRuns: 100 });
    });
});
