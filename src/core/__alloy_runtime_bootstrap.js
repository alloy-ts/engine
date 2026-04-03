// src/core/__alloy_runtime_bootstrap.js

// Evaluated by eval_fn before any plugin loads.
// Sets up the Alloy global namespace and event dispatcher.
globalThis.Alloy = globalThis.Alloy ?? {};
Alloy.plugins = {};

// Event dispatcher — called by emit_fn via __alloy_emit global
const __listeners = {};
globalThis.__alloy_emit = (event, payload) => {
    const parsed = typeof payload === 'string' ? JSON.parse(payload) : payload;
    const handlers = __listeners[event] || [];
    for (const fn of handlers) {
        try {
            fn(parsed);
        } catch (e) {
            console.error(`[alloy] error in listener for ${event}:`, e);
        }
    }
};

Alloy.on = (event, fn) => {
    __listeners[event] = __listeners[event] || [];
    __listeners[event].push(fn);
};

Alloy.off = (event, fn) => {
    if (!__listeners[event]) return;
    const idx = __listeners[event].indexOf(fn);
    if (idx !== -1) __listeners[event].splice(idx, 1);
};

// Result delivery bridge for WebViewAdapter
globalThis.__alloy_deliver_result = null;

globalThis.__alloy_dispatch_on_resolve = (path, importer) => {
    for (const name of Object.keys(Alloy.plugins)) {
        const p = Alloy.plugins[name];
        if (p.onResolve) {
            const res = p.onResolve({ path, importer });
            if (res) return JSON.stringify(res);
        }
    }
    return null;
};

globalThis.onStart = () => {
    for (const name of Object.keys(Alloy.plugins)) {
        const p = Alloy.plugins[name];
        if (p.onStart) p.onStart();
    }
};
