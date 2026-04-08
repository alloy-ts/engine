// src/core/plugins/async/alloy_async_shim.js
// ES5 strict compliance async runtime primitives

(function() {
    var __alloy_pending = {};
    var __alloy_next_task_id = 1;

    globalThis.__alloy_dispatch = function(op, args, continuation) {
        var taskId = __alloy_next_task_id++;
        __alloy_pending[taskId] = continuation;

        // Proxy to nativeInvoke
        try {
            var res = nativeInvoke(op, JSON.stringify(args));
            // In CPS, nativeInvoke might return immediately if sync,
            // or return taskId if scheduled
        } catch (e) {
            __alloy_reject(taskId, e.message);
        }
    };

    globalThis.__alloy_resolve = function(taskId, result) {
        var k = __alloy_pending[taskId];
        if (k) {
            delete __alloy_pending[taskId];
            k(null, result);
        }
    };

    globalThis.__alloy_reject = function(taskId, error) {
        var k = __alloy_pending[taskId];
        if (k) {
            delete __alloy_pending[taskId];
            k(error);
        }
    };
})();
