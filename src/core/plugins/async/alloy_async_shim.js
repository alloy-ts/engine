// alloy_async_shim.js — ES5 strict, no Promise/async/generator
var __alloy_pending = {};

function __alloy_dispatch(op, args, continuation) {
    var taskId = nativeInvoke(op, JSON.stringify(args));
    if (!taskId) {
        continuation({ message: 'nativeInvoke returned null for op: ' + op }, undefined);
        return 0;
    }
    __alloy_pending[taskId] = continuation;
    return taskId;
}

function __alloy_resolve(taskId, result) {
    var cont = __alloy_pending[taskId];
    if (!cont) {
        __alloy_unhandled_error('__alloy_resolve: unknown taskId ' + taskId);
        return;
    }
    delete __alloy_pending[taskId];
    cont(null, result);
}

function __alloy_reject(taskId, error) {
    var cont = __alloy_pending[taskId];
    if (!cont) {
        __alloy_unhandled_error('__alloy_reject: unknown taskId ' + taskId);
        return;
    }
    delete __alloy_pending[taskId];
    cont(error, undefined);
}
