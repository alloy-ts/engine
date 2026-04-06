/**
 * AlloyScript Runtime Bootstrap
 *
 * Evaluated once after engine init inside MicroQuickJS.
 * Sets up the Alloy global namespace, event system, and plugin registry.
 *
 * Authored in TypeScript; transpiled to ES5 by scripts/transpile.ts
 * before being embedded into the C binary.
 */

// ── Types (stripped at transpile time) ───────────────────────────────────

type Listener = (payload: unknown) => void;
type PluginAPI = Record<string, unknown>;

interface AlloyGlobal {
  version: string;
  plugins: Record<string, PluginAPI>;
  _listeners: Record<string, Listener[]>;
  on(event: string, fn: Listener): void;
  off(event: string, fn: Listener): void;
  emit(event: string, payload: unknown): void;
  _registerPlugin(name: string, api: PluginAPI): void;
}

// ── Bootstrap IIFE ────────────────────────────────────────────────────────

(function bootstrap(global: typeof globalThis) {
  "use strict";

  const existing = (global as Record<string, unknown>)["Alloy"] as Partial<AlloyGlobal> | undefined;

  const Alloy: AlloyGlobal = {
    version: "0.1.0",
    plugins: {},
    _listeners: {},

    on(event: string, fn: Listener): void {
      if (!Alloy._listeners[event]) Alloy._listeners[event] = [];
      Alloy._listeners[event]!.push(fn);
    },

    off(event: string, fn: Listener): void {
      const ls = Alloy._listeners[event];
      if (!ls) return;
      for (let i = ls.length - 1; i >= 0; i--) {
        if (ls[i] === fn) { ls.splice(i, 1); break; }
      }
    },

    emit(event: string, payload: unknown): void {
      const ls = Alloy._listeners[event];
      if (!ls) return;
      for (const fn of ls) {
        try { fn(payload); } catch { /* isolate listener errors */ }
      }
    },

    _registerPlugin(name: string, api: PluginAPI): void {
      if (Alloy.plugins[name]) throw new Error(`Plugin already registered: ${name}`);
      Alloy.plugins[name] = api;
    },

    // Carry over any plugins already registered before bootstrap ran
    ...existing,
  };

  // Internal event dispatcher — called by the C emit_fn via JS_Eval
  (global as Record<string, unknown>)["__alloy_emit"] = function (
    event: string,
    payload: unknown,
  ): void {
    Alloy.emit(event, payload);
  };

  (global as Record<string, unknown>)["Alloy"] = Alloy;
  (global as Record<string, unknown>)["globalThis"] = global;
  (global as Record<string, unknown>)["self"] = global;
})(globalThis);
