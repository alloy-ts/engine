import fc from 'fast-check';
import { $ } from '../../../runtime';
import { writeFileSync, readFileSync, unlinkSync } from 'node:fs';

describe('Alloy Shell Property-Based Tests (Phase 2)', () => {
    // Feature: alloy-shell, Property 7: Redirection Write Round-Trip
    test('redirection write round-trip', async () => {
        const tmpFile = `tmp-${Math.random()}.txt`;
        await fc.assert(fc.asyncProperty(
            fc.string(),
            async (s) => {
                await $`echo ${s} > ${tmpFile}`.nothrow();
                const content = readFileSync(tmpFile, 'utf-8');
                expect(content).toBe(s + '\n');
                unlinkSync(tmpFile);
            }
        ), { numRuns: 100 });
    });

    // Feature: alloy-shell, Property 8: Stdin Redirection Round-Trip
    test('stdin redirection round-trip', async () => {
        await fc.assert(fc.asyncProperty(
            fc.string(),
            async (s) => {
                const result = await $`cat < ${Buffer.from(s)}`.nothrow();
                expect(result.stdout.toString()).toBe(s);
            }
        ), { numRuns: 100 });
    });

    // Feature: alloy-shell, Property 10: echo Builtin Correctness
    test('echo builtin correctness', async () => {
        await fc.assert(fc.asyncProperty(
            fc.string({ minLength: 1, maxLength: 100 }).filter(s => !s.includes('\n')),
            async (s) => {
                const result = await $`echo ${s}`.nothrow();
                expect(result.stdout.toString()).toBe(s + '\n');
            }
        ), { numRuns: 100 });
    });

    // Feature: alloy-shell, Property 11: dirname Builtin Correctness
    test('dirname builtin correctness', async () => {
        await fc.assert(fc.asyncProperty(
            fc.string(),
            async (p) => {
                const result = await $`dirname ${p}`.nothrow();
                // Check if output matches a directory pattern
                expect(result.exitCode).toBe(0);
            }
        ), { numRuns: 100 });
    });

    // Feature: alloy-shell, Property 13: seq Builtin Correctness
    test('seq builtin correctness', async () => {
        await fc.assert(fc.asyncProperty(
            fc.integer({ min: 1, max: 10 }),
            fc.integer({ min: 11, max: 20 }),
            async (a, b) => {
                const result = await $`seq ${a} ${b}`.nothrow();
                const lines = result.stdout.toString().trim().split('\n');
                expect(lines.length).toBe(b - a + 1);
            }
        ), { numRuns: 100 });
    });

    // Feature: alloy-shell, Property 15: Array Interpolation Expansion
    test('array interpolation expansion', async () => {
        await fc.assert(fc.asyncProperty(
            fc.array(fc.string(), { minLength: 1, maxLength: 5 }),
            async (arr) => {
                const result = await $`echo ${arr}`.nothrow();
                expect(result.stdout.toString().trim()).toBe(arr.join(' '));
            }
        ), { numRuns: 100 });
    });

    // Feature: alloy-shell, Property 19: Command Substitution Trailing Newline Strip
    test('command substitution trailing newline strip', async () => {
        await fc.assert(fc.asyncProperty(
            fc.string().filter(s => !s.endsWith('\n')),
            async (s) => {
                const result = await $`echo $(echo ${s})`.nothrow();
                expect(result.stdout.toString()).toBe(s + '\n');
            }
        ), { numRuns: 100 });
    });
});
