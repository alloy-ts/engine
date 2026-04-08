// alloy_async_shim.js — ES5 Async Runtime Primitives
var __alloy_pending = {};
var __alloy_next_task_id = 1;

function __alloy_dispatch(op, args, continuation) {
    var taskId = __alloy_next_task_id++;
    __alloy_pending[taskId] = continuation;

    // Call C bridge
    var status = nativeInvoke(op, JSON.stringify(args), taskId);
    if (status === 0) {
        // Task failed to dispatch
        delete __alloy_pending[taskId];
        continuation(new Error("Async dispatch failed"), null);
    }
    return taskId;
}

function __alloy_resolve(taskId, result) {
    var k = __alloy_pending[taskId];
    if (k) {
        delete __alloy_pending[taskId];
        k(null, result);
    } else {
        __alloy_unhandled_error("Unknown taskId in __alloy_resolve: " + taskId);
    }
}

function __alloy_reject(taskId, error) {
    var k = __alloy_pending[taskId];
    if (k) {
        delete __alloy_pending[taskId];
        k(error, null);
    } else {
        __alloy_unhandled_error("Unknown taskId in __alloy_reject: " + taskId);
    }
}
