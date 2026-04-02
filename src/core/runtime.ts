(function () {
	const global = globalThis;

	// Internal bridge to the host (exposed via bind_global_mqjs)
	const AlloyInternal = global.AlloyInternal;

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
	};

	global.Alloy = Alloy;

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
