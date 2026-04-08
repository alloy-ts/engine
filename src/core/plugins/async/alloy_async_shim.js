// alloy_async_shim.js — ES5 strict, no Promise/async/generator
// Define primitives for CPS async execution in MicroQuickJS

var __alloy_pending = {};

/**
 * Dispatches an async operation to the C bridge.
 * @param {string} op Operation name (e.g., 'fetch', 'timer')
 * @param {any[]} args Arguments for the operation
 * @param {Function} continuation Callback (err, result) to resume execution
 */
function __alloy_dispatch(op, args, continuation) {
    // nativeInvoke is a C bridge global registered at runtime
    var taskId = nativeInvoke(op, JSON.stringify(args));
    if (!taskId) {
        continuation({ message: 'nativeInvoke returned null for op: ' + op }, undefined);
        return 0;
    }
    __alloy_pending[taskId] = continuation;
    return taskId;
}

/**
 * Success callback invoked by alloy_tick from C.
 */
function __alloy_resolve(taskId, result) {
    var cont = __alloy_pending[taskId];
    if (!cont) {
        __alloy_unhandled_error('__alloy_resolve: unknown taskId ' + taskId);
        return;
    }
    delete __alloy_pending[taskId];
    cont(null, result);
}

/**
 * Error callback invoked by alloy_tick from C.
 */
function __alloy_reject(taskId, error) {
    var cont = __alloy_pending[taskId];
    if (!cont) {
        __alloy_unhandled_error('__alloy_reject: unknown taskId ' + taskId);
        return;
    }
    delete __alloy_pending[taskId];
    cont(error, undefined);
}
