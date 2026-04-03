import { $ } from '../../../runtime';

describe('Alloy Shell Unit Tests (Phase 2)', () => {
    test('concurrency with Promise.all', async () => {
        const [r1, r2, r3] = await Promise.all([
            $`echo 1`.nothrow(),
            $`echo 2`.nothrow(),
            $`echo 3`.nothrow()
        ]);
        expect(r1.stdout.toString().trim()).toBe('1');
        expect(r2.stdout.toString().trim()).toBe('2');
        expect(r3.stdout.toString().trim()).toBe('3');
    });

    test('global setting $.nothrow()', async () => {
        $.nothrow();
        const res = await $`false`;
        expect(res.exitCode).toBe(1);
        $.throws(true); // reset
    });

    test('expression override of global setting', async () => {
        $.nothrow();
        try {
            await $`false`.throws(true);
            fail('Should have thrown');
        } catch (e: any) {
            expect(e.exitCode).toBe(1);
        }
        $.throws(true); // reset
    });

    test('env propagation via .env()', async () => {
        const res = await $`echo $FOO`.env({ FOO: 'bar' }).nothrow();
        expect(res.stdout.toString().trim()).toBe('bar');
    });

    test('cwd propagation via .cwd()', async () => {
        const res = await $`pwd`.cwd('/tmp').nothrow();
        expect(res.stdout.toString().trim()).toBe('/tmp');
    });

    test('json() consumer', async () => {
        const res = await $`echo '{"foo":"bar"}'`.json();
        expect(res).toEqual({ foo: 'bar' });
    });

    test('text() consumer', async () => {
        const res = await $`echo hello`.text();
        expect(res).toBe('hello\n');
    });
});
