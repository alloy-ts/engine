(function () {
	var global = globalThis;

	var AlloyInternal = global.AlloyInternal;

	var Alloy = {
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
				var transformed = Alloy.Transpiler.transformSync(code);
				// Call native secureEval binding
				return AlloyInternal.secureEval(transformed);
			} catch (e) {
				return { error: "transpile_failed", message: e.message };
			}
		},
		/**
		 * Spawns a process synchronously (Requirement 7.1).
		 */
		spawnSync: function (cmd, args, opts) {
			return AlloyInternal.spawnSync(cmd, args, opts);
		},
		/**
		 * Spawns a process asynchronously (Requirement 7.1).
		 */
		spawn: function (cmd, args, opts) {
			return new Promise(function (resolve, reject) {
				AlloyInternal.spawn(cmd, args, opts, function (err, res) {
					if (err) reject(err);
					else resolve(res);
				});
			});
		},
	};

	global.Alloy = Alloy;

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
