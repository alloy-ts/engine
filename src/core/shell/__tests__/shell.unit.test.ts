import { $ } from '../../runtime';

describe('Alloy Shell Unit Tests', () => {
    test('$ is defined globally', () => {
        expect($).toBeDefined();
    });

    test('simple echo command', async () => {
        const result = await $`echo "hello world"`.nothrow();
        expect(result.exitCode).toBe(0);
    });

    test('multi-line script', async () => {
        const result = await $`
            echo line 1
            echo line 2
        `.nothrow();
        expect(result.exitCode).toBe(0);
    });

    test('redirection (placeholder)', async () => {
        const result = await $`echo "data" > tmp.txt`.nothrow();
        expect(result.exitCode).toBe(0);
    });

    test('builtin command cd', async () => {
        const result = await $`cd /tmp`.nothrow();
        expect(result.exitCode).toBe(0);
    });
});
