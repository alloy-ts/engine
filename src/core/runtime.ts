(() => {
	const global = globalThis;

	// Internal bridge to the host (exposed via bind_global_mqjs)
	const AlloyInternal = global.AlloyInternal;

	const Alloy = {
		/**
		 * Binds a name and value to the MQJS global object.
		 */
		bind_global: (name, value) => {
			if (AlloyInternal && AlloyInternal.bindGlobal) {
				AlloyInternal.bindGlobal(name, value);
			}
		},

		/**
		 * Transpiles and evaluates user-supplied code securely.
		 */
		secureEval: (code) => {
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
		spawnSync: (cmd, args, opts) => AlloyInternal.spawnSync(cmd, args, opts),

		/**
		 * Spawns a process asynchronously.
		 */
		spawn: (cmd, args, opts) =>
			new Promise((resolve, reject) => {
				AlloyInternal.spawn(cmd, args, opts, (err, res) => {
					if (err) reject(err);
					else resolve(res);
				});
			}),

		/**
		 * Register event listener.
		 */
		on: (event, listener) => {
			AlloyInternal.on(event, listener);
		},

		/**
		 * Deregister event listener.
		 */
		off: (event, listener) => {
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
		getMmap: () => AlloyInternal.getMmap(),

		/**
		 * Returns the actual bytes of a specific memory region (gated).
		 */
		readMmap: (region) => AlloyInternal.readMmap(region),

		/**
		 * alloy:ffi re-export.
		 */
		ffi: undefined,

		/**
		 * Alloy Shell template tag.
		 */
		$: global.$ as ShellTag,
	};

	global.Alloy = Alloy;

	export interface ShellResult {
        stdout:   Buffer;
        stderr:   Buffer;
        exitCode: number;
        process:  any;
    }

	export interface ShellError extends Error {
        exitCode: number;
        stdout:   Buffer;
        stderr:   Buffer;
        message:  string;
    }

	export interface ShellExpression extends PromiseLike<ShellResult> {
        quiet():              ShellExpression;
        nothrow():            ShellExpression;
        throws(enabled: boolean): ShellExpression;
        env(vars: Record<string, string> | undefined): ShellExpression;
        cwd(path: string | undefined): ShellExpression;
        text():  Promise<string>;
        json():  Promise<unknown>;
        lines(): AsyncIterable<string>;
        blob():  Promise<Blob>;
    }

	export interface ShellTag {
        (strings: TemplateStringsArray, ...interps: unknown[]): ShellExpression;
        escape(str: string): string;
        braces(str: string): string[];
        nothrow(): void;
        throws(enabled: boolean): void;
        env(vars: Record<string, string> | undefined): void;
        cwd(path: string | undefined): void;
    }

	// --- BrowserAPI Proxies ---

	/**
	 * Proxy for window.fetch.
	 */
	global.fetch = (url, options) =>
		new Promise((resolve, reject) => {
			// Forward call via IPC through AlloyInternal.bridgeCall
			AlloyInternal.bridgeCall(
				"fetch",
				{ url: url, options: options },
				(err, res) => {
					if (err) reject(err);
					else
						resolve(new Response(res.body, { status: res.status, ok: res.ok }));
				},
			);
		});

	/**
	 * Proxy for crypto.subtle.
	 */
	global.crypto = {
		subtle: {
			digest: (algo, data) =>
				new Promise((resolve, reject) => {
					AlloyInternal.bridgeCall(
						"crypto.subtle.digest",
						{ algo: algo, data: data },
						(err, res) => {
							if (err) reject(err);
							else resolve(res);
						},
					);
				}),
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
			now: () => AlloyInternal.performanceNow(),
		};
	}
})();
