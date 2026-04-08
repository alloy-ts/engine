(() => {
	var global = globalThis;
	var AlloyInternal = global.AlloyInternal;

	var Alloy = {
		bind_global: (name, value) => {
			if (AlloyInternal?.bindGlobal) AlloyInternal.bindGlobal(name, value);
		},
		secureEval: (code) => {
			try {
				var transformed = Alloy.Transpiler.transformSync(code);
				return AlloyInternal.secureEval(transformed);
			} catch (e) {
				return { error: "transpile_failed", message: e.message };
			}
		},
		spawnSync: (cmd, args, opts) => AlloyInternal.spawnSync(cmd, args, opts),
		spawn: (cmd, args, opts) =>
			new Promise((resolve, reject) => {
				AlloyInternal.spawn(cmd, args, opts, (err, res) => {
					if (err) reject(err); else resolve(res);
				});
			}),
		on: (ev, cb) => AlloyInternal.on(ev, cb),
		emit: (ev, data) => AlloyInternal.emit(ev, data),
		getMmap: () => AlloyInternal.getMmap(),
		readMmap: (r) => AlloyInternal.readMmap(r),
		$: (() => {
			var $ = function (parts) {
				var interps = Array.prototype.slice.call(arguments, 1);
				var script = parts[0];
				for (var i = 0; i < interps.length; i++) {
					var val = interps[i];
					script += (typeof val === "string") ? $.escape(val) : val;
					script += parts[i + 1];
				}
				var expr = {
					_script: script, _throws: true,
					then: (onF, onR) => new Promise((resolve, reject) => {
						try {
							var res = AlloyInternal.shell(script);
							if (expr._throws && res.exitCode !== 0) reject(res); else resolve(res);
						} catch (e) { reject(e); }
					}).then(onF, onR),
					nothrow: () => { expr._throws = false; return expr; },
				};
				return expr;
			};
			$.escape = (str) => AlloyInternal.shellEscape(str);
			$.braces = (str) => {
				if (!str.includes("{")) return [str];
				var match = str.match(/\{([^}]+)\}/);
				if (!match) return [str];
				var parts = match[1].split(",");
				var result = [];
				for (var i = 0; i < parts.length; i++) result.push(str.replace(match[0], parts[i]));
				return result;
			};
			return $;
		})(),
	};

	global.Alloy = Alloy;
	global.$ = Alloy.$;

	global.fetch = (url, options) => {
		return __alloy_await({ __alloy_ipc: "fetch", args: [url, options] });
	};

	global.crypto = {
		subtle: {
			digest: (algo, data) => __alloy_await({ __alloy_ipc: "crypto.subtle.digest", args: [algo, data] }),
		},
		getRandomValues: (buf) => __alloy_await({ __alloy_ipc: "crypto.getRandomValues", args: [buf] }),
	};

	var R = function Response(body, options) {
		this._body = body;
		this.status = options.status || 200;
		this.ok = options.ok || (this.status >= 200 && this.status < 300);
	};
	R.prototype.text = function () { return Promise.resolve(this._body); };
	R.prototype.json = function () { return Promise.resolve(JSON.parse(this._body)); };
	global.Response = R;

	if (!global.performance) {
		global.performance = { now: () => AlloyInternal.performanceNow() };
	}
})();
