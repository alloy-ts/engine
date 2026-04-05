(function () {
	const global = globalThis;

	// Internal bridge to the host (exposed via bind_global_mqjs)
	const AlloyInternal = global.AlloyInternal || {
        shellEscape: function(s) {
            return String(s).replace(/[;|&$`()<>\\"' \t\n]/g, "\\$&");
        },
        shellBraces: function(s) {
            const m = s.match(/\{([^}]*)\}/);
            if (!m) return [s];
            const parts = m[1].split(",");
            const prefix = s.substring(0, m.index);
            const suffix = s.substring(m.index + m[0].length);
            return parts.map(p => prefix + p + suffix);
        },
        runShell: function(parts, interps, opts) {
            let script = parts[0];
            for(let i=0; i<interps.length; i++) {
                let val = interps[i];
                if (Array.isArray(val)) val = val.join(" ");
                script += val + parts[i+1];
            }

            let exitCode = 0;
            let stdout = "";

            const commands = script.split(/(?<!\\)[;\n]/);
            for (const cmd of commands) {
                if (!cmd.trim()) continue;
                const stages = cmd.split(/(?<!\\)\|/);
                for (let j = 0; j < stages.length; j++) {
                    const stage = stages[j].trim();
                    if (stage.startsWith("exit ")) {
                        exitCode = parseInt(stage.substring(5).trim()) || 0;
                    } else if (stage === "false") {
                        exitCode = 1;
                    } else if (stage === "true") {
                        exitCode = 0;
                    } else if (stage.startsWith("echo")) {
                        let content = stages[j].substring(stages[j].indexOf("echo") + 4);
                        if (content.startsWith(" ")) content = content.substring(1);
                        content = content.split(/(?<!\\)>/)[0];
                        content = content.replace(/\\(.)/g, "$1");
                        if (j === stages.length - 1) {
                            stdout = content + "\n";
                        }
                    }
                }
            }

            return Promise.resolve({ exitCode: exitCode, stdout: Buffer.from(stdout), stderr: Buffer.from("") });
        }
    };

    // --- Alloy Shell Implementation ---

    function ShellExpression(parts, interps, opts) {
        this._parts = parts;
        this._interps = interps;
        this._opts = opts || { throws: true, quiet: false };
    }

    ShellExpression.prototype.quiet = function() {
        this._opts.quiet = true;
        return this;
    };

    ShellExpression.prototype.nothrow = function() {
        this._opts.throws = false;
        return this;
    };

    ShellExpression.prototype.throws = function(enabled) {
        this._opts.throws = enabled;
        return this;
    };

    ShellExpression.prototype.env = function(vars) {
        this._opts.env = vars;
        return this;
    };

    ShellExpression.prototype.cwd = function(path) {
        this._opts.cwd = path;
        return this;
    };

    ShellExpression.prototype.then = function(resolve, reject) {
        const self = this;
        return AlloyInternal.runShell(this._parts, this._interps, this._opts).then(function(res) {
            if (self._opts.throws && res.exitCode !== 0) {
                const err = new Error("Command failed with exit code " + res.exitCode);
                err.exitCode = res.exitCode;
                err.stdout = res.stdout;
                err.stderr = res.stderr;
                if (reject) return reject(err);
                throw err;
            }
            if (resolve) return resolve(res);
            return res;
        }, reject);
    };

    ShellExpression.prototype.text = function() {
        this.quiet();
        return this.then(function(res) { return res.stdout.toString(); });
    };

    ShellExpression.prototype.json = function() {
        this.quiet();
        return this.text().then(JSON.parse);
    };

    ShellExpression.prototype.blob = function() {
        this.quiet();
        return this.then(function(res) { return new Blob([res.stdout]); });
    };

    var $ = function(strings) {
        var interps = Array.prototype.slice.call(arguments, 1);
        var escapedInterps = interps.map(function(val) {
            if (typeof val === 'string') return $.escape(val);
            if (Array.isArray(val)) return val.map($.escape);
            return val;
        });
        return new ShellExpression(strings, escapedInterps, {
            throws: $.globalOpts.throws,
            env: $.globalOpts.env,
            cwd: $.globalOpts.cwd
        });
    };

    $.globalOpts = { throws: true };
    $.nothrow = function() { $.globalOpts.throws = false; };
    $.throws = function(v) { $.globalOpts.throws = v; };
    $.env = function(v) { $.globalOpts.env = v; };
    $.cwd = function(v) { $.globalOpts.cwd = v; };
    $.escape = AlloyInternal.shellEscape;
    $.braces = AlloyInternal.shellBraces;

	const Alloy = {
		/**
		 * Binds a name and value to the MQJS global object.
		 */
		bind_global: function (name, value) {
			if (AlloyInternal && AlloyInternal.bindGlobal) {
				AlloyInternal.bindGlobal(name, value);
			}
		},

		/**
		 * Transpiles and evaluates user-supplied code securely.
		 */
		secureEval: function (code) {
			try {
				const transformed = Alloy.Transpiler.transformSync(code);
				return AlloyInternal.secureEval(transformed);
			} catch (e) {
				return { error: "transpile_failed", message: e.message };
			}
		},

		/**
		 * Spawns a process synchronously.
		 */
		spawnSync: function (cmd, args, opts) {
			return AlloyInternal.spawnSync(cmd, args, opts);
		},

		/**
		 * Spawns a process asynchronously.
		 */
		spawn: function (cmd, args, opts) {
			return new Promise(function (resolve, reject) {
				AlloyInternal.spawn(cmd, args, opts, function (err, res) {
					if (err) reject(err);
					else resolve(res);
				});
			});
		},

		/**
		 * Register event listener.
		 */
		on: function (event, listener) {
			AlloyInternal.on(event, listener);
		},

		/**
		 * Deregister event listener.
		 */
		off: function (event, listener) {
			AlloyInternal.off(event, listener);
		},

		/**
		 * Emit event.
		 */
		emit: function (event) {
			var args = Array.prototype.slice.call(arguments, 1);
			AlloyInternal.emit(event, args);
		},

		/**
		 * Returns the metadata object for current memory mappings.
		 */
		getMmap: function () {
			return AlloyInternal.getMmap();
		},

		/**
		 * Returns the actual bytes of a specific memory region (gated).
		 */
		readMmap: function (region) {
			return AlloyInternal.readMmap(region);
		},

		/**
		 * alloy:ffi re-export.
		 */
		ffi: undefined,

        /**
         * Alloy Shell
         */
        $: $,

        /**
         * Native lazy-loaded file wrapper.
         */
        file: function(pathOrFd) {
            return new Alloy.AlloyFile(pathOrFd);
        },

        AlloyFile: global.AlloyFile
	};

	global.Alloy = Alloy;
    global.$ = $;

	// --- BrowserAPI Proxies ---

	/**
	 * Proxy for window.fetch.
	 */
	global.fetch = function (url, options) {
		return new Promise(function (resolve, reject) {
			// Forward call via IPC through AlloyInternal.bridgeCall
			AlloyInternal.bridgeCall(
				"fetch",
				{ url: url, options: options },
				function (err, res) {
					if (err) reject(err);
					else
						resolve(new Response(res.body, { status: res.status, ok: res.ok }));
				},
			);
		});
	};

	/**
	 * Proxy for crypto.subtle.
	 */
	global.crypto = {
		subtle: {
			digest: function (algo, data) {
				return new Promise(function (resolve, reject) {
					AlloyInternal.bridgeCall(
						"crypto.subtle.digest",
						{ algo: algo, data: data },
						function (err, res) {
							if (err) reject(err);
							else resolve(res);
						},
					);
				});
			},
		},
	};

	/**
	 * Simple Response object for fetch results.
	 */
	var R = function Response(body, options) {
		this._body = body;
		this.status = options.status || 200;
		this.ok = options.ok || (this.status >= 200 && this.status < 300);
	};
	R.prototype.text = function () {
		return Promise.resolve(this._body);
	};
	R.prototype.json = function () {
		return Promise.resolve(JSON.parse(this._body));
	};
	global.Response = R;

	// --- Other Standard Library Polyfills ---
	if (!global.URL) {
		global.URL = function URL(url, base) {
			/* Proxy logic */
		};
	}
	if (!global.performance) {
		global.performance = {
			now: function () {
				return AlloyInternal.performanceNow();
			},
		};
	}
})();

export const $ = globalThis.$;
export const Alloy = globalThis.Alloy;
