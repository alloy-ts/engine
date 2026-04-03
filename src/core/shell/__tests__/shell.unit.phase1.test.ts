import { $ } from '../../../runtime';

describe('Alloy Shell Unit Tests (Phase 1)', () => {
    test('echo builtin', async () => {
        const res = await $`echo hello world`.nothrow();
        expect(res.stdout.toString()).toBe('hello world\n');
        expect(res.exitCode).toBe(0);
    });

    test('pwd and cd builtins', async () => {
        const initial = (await $`pwd`.nothrow()).stdout.toString().trim();
        await $`cd /tmp && pwd`.nothrow();
        const current = (await $`pwd`.nothrow()).stdout.toString().trim();
        expect(current).toBe(initial); // cd only affects subsequent commands in the same ShellExpression
    });

    test('cd within a script', async () => {
        const res = await $`cd /tmp; pwd`.nothrow();
        expect(res.stdout.toString().trim()).toBe('/tmp');
    });

    test('true and false builtins', async () => {
        expect((await $`true`.nothrow()).exitCode).toBe(0);
        expect((await $`false`.nothrow()).exitCode).toBe(1);
    });

    test('redirection 2>&1', async () => {
        const res = await $`echo stdout; echo stderr >&2`.nothrow();
        // Here we'd check if stderr is separated. For 2>&1:
        const merged = await $`echo stdout; echo stderr >&2 2>&1`.nothrow();
        expect(merged.stdout.toString()).toContain('stdout');
        expect(merged.stdout.toString()).toContain('stderr');
    });

    test('redirection with AlloyFile', async () => {
        // Mock Alloy.file for testing
    });
});
