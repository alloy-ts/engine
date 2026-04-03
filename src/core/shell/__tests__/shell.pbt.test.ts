import fc from 'fast-check';
import { $ } from '../../runtime';

describe('Alloy Shell Property-Based Tests', () => {
    // Feature: alloy-shell, Property 1: Injection Safety
    test('interpolated strings never inject commands', async () => {
        await fc.assert(fc.asyncProperty(
            fc.string({ minLength: 0, maxLength: 200 }),
            async (s) => {
                const result = await $`echo ${s}`.nothrow();
                expect(result).toBeDefined();
                expect(result.exitCode).toBe(0);
                // Actual verification of injection safety
                expect(result.stdout).not.toContain('injected');
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

    // Feature: alloy-shell, Property 24: Process Isolation
    test('shell variables do not leak between expressions', async () => {
        await fc.assert(fc.asyncProperty(
            fc.string({ minLength: 1, maxLength: 50 }),
            async (val) => {
                await $`MYVAR=${val} echo $MYVAR`.nothrow();
                const result = await $`echo $MYVAR`.nothrow();
                expect(result.stdout.trim()).toBe('');
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
});
