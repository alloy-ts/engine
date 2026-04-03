// shell.js — alloy:shell JS entry point

class ShellExpression {
    #strings; #values; #opts;

    constructor(strings, values) {
        this.#strings = strings;
        this.#values = values;
        this.#opts = {
            nothrow: false,
            env: {},
            cwd: '.',
        };
    }

    #buildPayload() {
        let cmd = '';
        for (let i = 0; i < this.#strings.length; i++) {
            cmd += this.#strings[i];
            if (i < this.#values.length) {
                cmd += __alloy_shell_escape(String(this.#values[i]));
            }
        }
        return JSON.stringify({
            cmd,
            env: this.#opts.env,
            cwd: this.#opts.cwd,
        });
    }

    then(resolve, reject) {
        const payload = this.#buildPayload();
        const resultJson = __alloy_shell_exec(payload);
        const result = JSON.parse(resultJson);
        if (result.error) return reject(new Error(result.error));
        if (result.exitCode !== 0 && !this.#opts.nothrow) {
            return reject(new Error(`Command exited with code ${result.exitCode}`));
        }
        return resolve(result);
    }

    nothrow() { this.#opts.nothrow = true; return this; }
}

export function $(strings, ...values) {
    return new ShellExpression(strings, values);
}

$.escape = (s) => __alloy_shell_escape(s);

globalThis.$ = $;
Alloy.$ = $;

export default { $, escape: $.escape };
