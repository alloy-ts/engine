(function () {
	var global = globalThis;
	var AlloyInternal = global.AlloyInternal;

	var Alloy = {
		Transpiler: function (options) {
			this.options = options || {};
		},
		bind_global: function (name, value) {
			if (AlloyInternal && AlloyInternal.bindGlobal) {
				AlloyInternal.bindGlobal(name, value);
			}
		},
		secureEval: function (code) {
			try {
				var transpiler = new Alloy.Transpiler({ loader: "tsx" });
				var transformed = transpiler.transformSync(code);
				return AlloyInternal.secureEval(transformed);
			} catch (e) {
				return { error: "transpile_failed", message: e.message };
			}
		},
		spawnSync: function (cmd, args, opts) {
			return AlloyInternal.spawnSync(cmd, args, opts);
		},
		spawn: function (cmd, args, opts) {
			return new Promise(function (resolve, reject) {
				AlloyInternal.spawn(cmd, args, opts, function (err, res) {
					if (err) reject(err);
					else resolve(res);
				});
			});
		},
	};

	Alloy.Transpiler.prototype.transformSync = function (code, loader) {
		return code;
	};

	global.Alloy = Alloy;

	var browserAPIs = [
		"fetch",
		"crypto.subtle.digest",
		"crypto.getRandomValues",
		"URL",
		"URLSearchParams",
		"TextEncoder",
		"TextDecoder",
		"atob",
		"btoa",
		"performance.now",
		"performance.mark",
		"performance.measure",
		"ReadableStream",
		"WritableStream",
		"TransformStream",
		"Blob",
		"File",
		"FormData",
		"Headers",
		"Request",
		"Response",
		"AbortController",
		"AbortSignal",
		"EventTarget",
		"CustomEvent",
		"MessageChannel",
		"MessagePort",
		"BroadcastChannel",
		"WebSocket",
		"indexedDB",
		"caches",
		"setInterval",
		"clearInterval",
		"queueMicrotask",
		"reportError",
		"alert",
		"confirm",
		"prompt",
		"structuredClone",
		"Worker",
	];

	function createProxy(api) {
		return function () {
			var args = Array.prototype.slice.call(arguments);
			return new Promise(function (resolve, reject) {
				AlloyInternal.bridgeCall(api, args, function (err, res) {
					if (err) reject(err);
					else resolve(res);
				});
			});
		};
	}

	browserAPIs.forEach(function (api) {
		var parts = api.split(".");
		var target = global;
		for (var i = 0; i < parts.length - 1; i++) {
			if (!target[parts[i]]) target[parts[i]] = {};
			target = target[parts[i]];
		}
		var name = parts[parts.length - 1];
		if (!target[name]) {
			target[name] = createProxy(api);
		}
	});

	global.window = global;
	global.self = global;
	global.global = global;

	global.console = {
		log: function () {
			if (AlloyInternal && AlloyInternal.log) {
				AlloyInternal.log("info", Array.prototype.slice.call(arguments));
			} else {
				print(Array.prototype.slice.call(arguments).join(" "));
			}
		},
		warn: function () {
			if (AlloyInternal && AlloyInternal.log) {
				AlloyInternal.log("warn", Array.prototype.slice.call(arguments));
			} else {
				print("WARN:", Array.prototype.slice.call(arguments).join(" "));
			}
		},
		error: function () {
			if (AlloyInternal && AlloyInternal.log) {
				AlloyInternal.log("error", Array.prototype.slice.call(arguments));
			} else {
				print("ERROR:", Array.prototype.slice.call(arguments).join(" "));
			}
		},
	};

	global.setTimeout = function (fn, delay) {
		if (AlloyInternal && AlloyInternal.setTimeout) {
			return AlloyInternal.setTimeout(fn, delay);
		}
		return 0;
	};

	global.clearTimeout = function (id) {
		if (AlloyInternal && AlloyInternal.clearTimeout) {
			AlloyInternal.clearTimeout(id);
		}
	};

	global.__alloy_run_coroutine = function (gen) {
		return new Promise(function (resolve, reject) {
			function step(val) {
				try {
					var res = gen.next(val);
					if (res.done) {
						resolve(res.value);
					} else {
						var yielded = res.value;
						if (yielded && typeof yielded.then === "function") {
							yielded.then(step, reject);
						} else {
							step(yielded);
						}
					}
				} catch (e) {
					reject(e);
				}
			}
			step();
		});
	};

	/* Define alloy:ffi shim */
	var ffi = global.__alloy_ffi || {};
	ffi.suffix =
		global.process && global.process.platform === "win32"
			? "dll"
			: global.process && global.process.platform === "darwin"
				? "dylib"
				: "so";

	ffi.FFIType = {
		buffer: 0,
		cstring: 1,
		function: 2,
		fn: 2,
		callback: 2,
		ptr: 3,
		pointer: 3,
		"void*": 3,
		"char*": 3,
		i8: 4,
		i16: 5,
		i32: 6,
		int: 6,
		i64: 7,
		i64_fast: 8,
		u8: 9,
		u16: 10,
		u32: 11,
		u64: 12,
		u64_fast: 13,
		f32: 14,
		f64: 15,
		bool: 16,
		char: 17,
		napi_env: 18,
		napi_value: 19,
	};

	global["alloy:ffi"] = ffi;
})();
