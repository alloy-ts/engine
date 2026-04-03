export const ASYNC_POLYFILL = `
(function(g) {
  if (g.Promise) return;

  // Minimal Promise polyfill that supports the __alloy_await mechanism
  g.Promise = function Promise(executor) {
    this._status = 'pending';
    this._value = undefined;
    this._handlers = [];

    var self = this;
    var resolve = function(val) {
      if (self._status !== 'pending') return;
      self._status = 'fulfilled';
      self._value = val;
      self._handlers.forEach(function(h) { h.onFulfilled(val); });
    };
    var reject = function(reason) {
      if (self._status !== 'pending') return;
      self._status = 'rejected';
      self._value = reason;
      self._handlers.forEach(function(h) { h.onRejected(reason); });
    };

    try {
      executor(resolve, reject);
    } catch (e) {
      reject(e);
    }
  };

  g.Promise.prototype.then = function(onFulfilled, onRejected) {
    var self = this;
    return new g.Promise(function(resolve, reject) {
      var handle = function() {
        try {
          if (self._status === 'fulfilled') {
            resolve(onFulfilled ? onFulfilled(self._value) : self._value);
          } else {
            reject(onRejected ? onRejected(self._value) : self._value);
          }
        } catch (e) {
          reject(e);
        }
      };
      if (self._status === 'pending') {
        self._handlers.push({ onFulfilled: handle, onRejected: handle });
      } else {
        handle();
      }
    });
  };

  g.structuredClone = function(val) {
    return JSON.parse(JSON.stringify(val));
  };

  // Other polyfills...
})(globalThis);
`;

export const BROWSER_API_POLYFILLS = `
(function(g) {
    // Mirror of ALLOY_BROWSER_API_TABLE
    var apis = [
        "fetch", "Request", "Response", "Headers", "AbortController", "AbortSignal",
        "URL", "URLSearchParams", "Worker", "MessagePort", "MessageChannel", "BroadcastChannel",
        "ReadableStream", "WritableStream", "TransformStream", "ByteLengthQueuingStrategy", "CountQueuingStrategy",
        "Blob", "File", "FormData", "WebSocket", "TextEncoder", "TextDecoder",
        "performance.now", "performance.mark", "performance.measure",
        "EventTarget", "Event", "ErrorEvent", "CloseEvent", "MessageEvent",
        "indexedDB", "caches"
    ];

    // Most are already bound by the C loop.
    // Here we can add extra JS-side logic if needed.
})(globalThis);
`;
