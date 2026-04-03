(() => {
	const global = globalThis as any;

	// Polyfill for __alloy_deferred
	global.__alloy_deferred = (executor: any) => {
		let resolve, reject;
		const promise = {
			then: function (onFulfilled: any) {
				this._onFulfilled = onFulfilled;
				return this;
			},
			catch: function (onRejected: any) {
				this._onRejected = onRejected;
				return this;
			},
			_onFulfilled: null as any,
			_onRejected: null as any,
			_state: "pending",
			_value: null as any,
		};

		resolve = (value: any) => {
			promise._state = "fulfilled";
			promise._value = value;
			if (promise._onFulfilled) promise._onFulfilled(value);
		};

		reject = (error: any) => {
			promise._state = "rejected";
			promise._value = error;
			if (promise._onRejected) promise._onRejected(error);
		};

		executor(resolve, reject);
		return promise;
	};
})();
