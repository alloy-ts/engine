/**
 * AlloyScript Web API Shims
 *
 * Injected into MicroQuickJS after bootstrap. Each shim forwards calls
 * to the hidden alloy:web sidecar via __alloy_web_invoke (registered by
 * the alloy:web C plugin). The C host drives the call synchronously.
 *
 * Authored in TypeScript; transpiled to ES5 by scripts/transpile.ts.
 * When ALLOY_ENABLE_WEB=OFF this file is never embedded and LTO removes
 * all references to the sidecar.
 */

// ── Types (stripped at transpile time) ───────────────────────────────────

type InvokeResult = Record<string, unknown> & { error?: string };

interface HeadersMap {
  _map: Record<string, string>;
  get(key: string): string | null;
  set(key: string, value: string): void;
  has(key: string): boolean;
  append(key: string, value: string): void;
  entries(): Array<[string, string]>;
}

interface AlloyResponse {
  ok: boolean;
  status: number;
  statusText: string;
  headers: HeadersMap;
  url: string;
  text(): string;
  json(): unknown;
  arrayBuffer(): unknown;
}

// ── Bootstrap IIFE ────────────────────────────────────────────────────────

(function installWebAPIs(global: typeof globalThis) {
  "use strict";

  // Provided by the alloy:web C plugin. Falls back to a stub when the
  // sidecar is unavailable so the shim file can be loaded unconditionally.
  const _rawInvoke = (global as Record<string, unknown>)["__alloy_web_invoke"] as
    | ((method: string, argsJson: string, capId: string) => string)
    | undefined;

  function invoke(method: string, args: unknown, capId = ""): InvokeResult {
    if (!_rawInvoke) return { error: "alloy:web not available" };
    try {
      return JSON.parse(_rawInvoke(method, JSON.stringify(args), capId)) as InvokeResult;
    } catch {
      return { error: "invalid IPC response" };
    }
  }

  // ── Headers ─────────────────────────────────────────────────────────────

  function makeHeaders(init?: Record<string, string> | HeadersMap): HeadersMap {
    const map: Record<string, string> = {};
    if (init) {
      const src = "_map" in init ? (init as HeadersMap)._map : init;
      for (const k of Object.keys(src)) {
        map[k.toLowerCase()] = String((src as Record<string, string>)[k]);
      }
    }
    return {
      _map: map,
      get: (k: string) => map[k.toLowerCase()] ?? null,
      set: (k: string, v: string) => { map[k.toLowerCase()] = v; },
      has: (k: string) => k.toLowerCase() in map,
      append: (k: string, v: string) => {
        const lk = k.toLowerCase();
        map[lk] = map[lk] ? `${map[lk]}, ${v}` : v;
      },
      entries: () => Object.keys(map).map((k): [string, string] => [k, map[k]!]),
    };
  }

  // ── Response ─────────────────────────────────────────────────────────────

  function makeResponse(raw: InvokeResult): AlloyResponse {
    const body = raw["body"] ?? null;
    const status = (raw["status"] as number) ?? 0;
    return {
      ok: (raw["ok"] as boolean) ?? (status >= 200 && status < 300),
      status,
      statusText: (raw["statusText"] as string) ?? "",
      headers: makeHeaders((raw["headers"] as Record<string, string>) ?? {}),
      url: (raw["url"] as string) ?? "",
      text: () => String(body ?? ""),
      json: () => {
        try { return JSON.parse(String(body ?? "")); }
        catch (e) { throw new Error(`Response.json(): ${e}`); }
      },
      arrayBuffer: () => body,
    };
  }

  // ── fetch ─────────────────────────────────────────────────────────────────

  (global as Record<string, unknown>)["fetch"] = function fetch(
    input: string | { url: string },
    init?: { method?: string; headers?: Record<string, string>; body?: unknown; __capId?: string },
  ): AlloyResponse {
    const url = typeof input === "string" ? input : input.url;
    const result = invoke("fetch", {
      url,
      method: init?.method ?? "GET",
      headers: init?.headers ?? {},
      body: init?.body ?? null,
    }, init?.__capId ?? "");
    if (result.error) throw new Error(`fetch: ${result.error}`);
    return makeResponse(result);
  };

  // ── Request / Response / Headers ──────────────────────────────────────────

  (global as Record<string, unknown>)["Headers"] = makeHeaders;
  (global as Record<string, unknown>)["Response"] = makeResponse;
  (global as Record<string, unknown>)["Request"] = function Request(
    url: string,
    init?: { method?: string; headers?: Record<string, string>; body?: unknown },
  ) {
    return { url, method: init?.method ?? "GET", headers: makeHeaders(init?.headers), body: init?.body };
  };

  // ── URL ───────────────────────────────────────────────────────────────────

  (global as Record<string, unknown>)["URL"] = function URL(href: string, base?: string) {
    const result = invoke("url_parse", { href, base: base ?? "" });
    if (result.error) throw new TypeError(`Invalid URL: ${href}`);
    return { ...result, toString: () => (result["href"] as string) ?? href };
  };

  // ── URLSearchParams ───────────────────────────────────────────────────────

  (global as Record<string, unknown>)["URLSearchParams"] = function URLSearchParams(
    init?: string | Record<string, string>,
  ) {
    const params: Array<[string, string]> = [];
    if (typeof init === "string") {
      for (const pair of init.replace(/^\?/, "").split("&")) {
        const [k, v = ""] = pair.split("=");
        if (k) params.push([decodeURIComponent(k), decodeURIComponent(v)]);
      }
    } else if (init) {
      for (const k of Object.keys(init)) params.push([k, init[k]!]);
    }
    return {
      get: (k: string) => params.find(([pk]) => pk === k)?.[1] ?? null,
      set: (k: string, v: string) => {
        const i = params.findIndex(([pk]) => pk === k);
        if (i >= 0) params[i] = [k, v]; else params.push([k, v]);
      },
      has: (k: string) => params.some(([pk]) => pk === k),
      append: (k: string, v: string) => params.push([k, v]),
      toString: () => params.map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`).join("&"),
      entries: () => params[Symbol.iterator](),
    };
  };

  // ── TextEncoder / TextDecoder ─────────────────────────────────────────────

  (global as Record<string, unknown>)["TextEncoder"] = function TextEncoder() {};
  (TextEncoder as unknown as { prototype: { encode(s: string): number[] } }).prototype.encode =
    function (str: string): number[] {
      const r = invoke("text_encode", { text: str });
      return (r["bytes"] as number[]) ?? [];
    };

  (global as Record<string, unknown>)["TextDecoder"] = function TextDecoder(encoding = "utf-8") {
    this.encoding = encoding;
  };
  (TextDecoder as unknown as { prototype: { decode(b: unknown): string } }).prototype.decode =
    function (bytes: unknown): string {
      const r = invoke("text_decode", { bytes, encoding: this.encoding });
      return (r["text"] as string) ?? "";
    };

  // ── AbortController / AbortSignal ─────────────────────────────────────────

  (global as Record<string, unknown>)["AbortController"] = function AbortController() {
    let aborted = false;
    const listeners: Array<() => void> = [];
    this.signal = {
      aborted: false,
      addEventListener: (_: string, fn: () => void) => listeners.push(fn),
      removeEventListener: (_: string, fn: () => void) => {
        const i = listeners.lastIndexOf(fn);
        if (i >= 0) listeners.splice(i, 1);
      },
    };
    this.abort = () => {
      if (aborted) return;
      aborted = true;
      this.signal.aborted = true;
      for (const fn of listeners) { try { fn(); } catch { /* */ } }
    };
  };

  // ── Blob / File ───────────────────────────────────────────────────────────

  (global as Record<string, unknown>)["Blob"] = function Blob(
    parts: unknown[] = [],
    opts?: { type?: string },
  ) {
    this._parts = parts;
    this.type = opts?.type ?? "";
    this.size = parts.reduce((n, p) => n + String(p ?? "").length, 0);
    this.text = () => parts.join("");
    this.arrayBuffer = () => {
      const r = invoke("blob_to_arraybuffer", { data: parts.join("") });
      return (r["bytes"] as unknown[]) ?? [];
    };
  };

  (global as Record<string, unknown>)["File"] = function File(
    parts: unknown[],
    name: string,
    opts?: { type?: string; lastModified?: number },
  ) {
    Blob.call(this, parts, opts);
    this.name = name;
    this.lastModified = opts?.lastModified ?? Date.now();
  };

  // ── FormData ──────────────────────────────────────────────────────────────

  (global as Record<string, unknown>)["FormData"] = function FormData() {
    const entries: Array<{ name: string; value: unknown; filename?: string }> = [];
    this.append = (name: string, value: unknown, filename?: string) =>
      entries.push({ name, value, filename });
    this.get = (name: string) => entries.find((e) => e.name === name)?.value ?? null;
    this.has = (name: string) => entries.some((e) => e.name === name);
    this.entries = () => entries.slice();
  };

  // ── Streams ───────────────────────────────────────────────────────────────

  (global as Record<string, unknown>)["ReadableStream"] = function ReadableStream(
    _source?: unknown,
  ) {
    const chunks: unknown[] = [];
    this._push = (chunk: unknown) => chunks.push(chunk);
    this.getReader = () => ({
      read: () =>
        chunks.length > 0
          ? { value: chunks.shift(), done: false }
          : { value: undefined, done: true },
      cancel: () => { chunks.length = 0; },
    });
  };

  (global as Record<string, unknown>)["WritableStream"] = function WritableStream(
    sink?: { write?(c: unknown): void; close?(): void },
  ) {
    this.getWriter = () => ({
      write: (chunk: unknown) => sink?.write?.(chunk),
      close: () => sink?.close?.(),
    });
  };

  (global as Record<string, unknown>)["TransformStream"] = function TransformStream(
    transformer?: { transform?(c: unknown): unknown; flush?(): void },
  ) {
    const rs = new ReadableStream();
    this.readable = rs;
    this.writable = new WritableStream({
      write: (chunk: unknown) => {
        const out = transformer?.transform ? transformer.transform(chunk) : chunk;
        if (out !== undefined) rs._push(out);
      },
      close: () => transformer?.flush?.(),
    });
  };

  (global as Record<string, unknown>)["ByteLengthQueuingStrategy"] = function (
    init: { highWaterMark: number },
  ) {
    this.highWaterMark = init.highWaterMark;
    this.size = (chunk: ArrayBufferView) => chunk.byteLength;
  };

  (global as Record<string, unknown>)["CountQueuingStrategy"] = function (
    init: { highWaterMark: number },
  ) {
    this.highWaterMark = init.highWaterMark;
    this.size = () => 1;
  };

  // ── WebSocket ─────────────────────────────────────────────────────────────

  const WS = function WebSocket(url: string, protocols?: string | string[]) {
    this.url = url;
    this.protocols = protocols ?? [];
    this.readyState = 0;
    this.onopen = null;
    this.onmessage = null;
    this.onerror = null;
    this.onclose = null;

    const result = invoke("websocket_connect", { url, protocols });
    if (result.error) {
      this.readyState = 3;
      this.onerror?.({ message: result.error });
    } else {
      this._connId = result["connId"];
      this.readyState = 1;
      this.onopen?.({});
    }

    this.send = (data: unknown) => {
      if (this.readyState !== 1) throw new Error("WebSocket not open");
      invoke("websocket_send", { connId: this._connId, data });
    };
    this.close = (code?: number, reason?: string) => {
      invoke("websocket_close", { connId: this._connId, code, reason });
      this.readyState = 3;
      this.onclose?.({ code: code ?? 1000, reason: reason ?? "" });
    };
  };
  WS.CONNECTING = 0; WS.OPEN = 1; WS.CLOSING = 2; WS.CLOSED = 3;
  (global as Record<string, unknown>)["WebSocket"] = WS;

  // ── crypto ────────────────────────────────────────────────────────────────

  (global as Record<string, unknown>)["crypto"] = {
    getRandomValues: (arr: number[]) => {
      const r = invoke("crypto_random", { length: arr.length });
      const bytes = (r["bytes"] as number[]) ?? [];
      for (let i = 0; i < arr.length; i++) arr[i] = bytes[i] ?? 0;
      return arr;
    },
    subtle: {
      digest: (algo: string, data: unknown) =>
        invoke("crypto_digest", { algorithm: algo, data }),
      generateKey: (algo: unknown, extractable: boolean, usages: string[]) =>
        invoke("crypto_generate_key", { algorithm: algo, extractable, usages }),
      encrypt: (algo: unknown, key: unknown, data: unknown) =>
        invoke("crypto_encrypt", { algorithm: algo, key, data }),
      decrypt: (algo: unknown, key: unknown, data: unknown) =>
        invoke("crypto_decrypt", { algorithm: algo, key, data }),
      sign: (algo: unknown, key: unknown, data: unknown) =>
        invoke("crypto_sign", { algorithm: algo, key, data }),
      verify: (algo: unknown, key: unknown, sig: unknown, data: unknown) =>
        invoke("crypto_verify", { algorithm: algo, key, signature: sig, data }),
    },
  };

  // ── EventTarget / Event ───────────────────────────────────────────────────

  (global as Record<string, unknown>)["EventTarget"] = function EventTarget() {
    const listeners: Record<string, Array<(e: unknown) => void>> = {};
    this.addEventListener = (type: string, fn: (e: unknown) => void) => {
      (listeners[type] ??= []).push(fn);
    };
    this.removeEventListener = (type: string, fn: (e: unknown) => void) => {
      const ls = listeners[type];
      if (!ls) return;
      const i = ls.lastIndexOf(fn);
      if (i >= 0) ls.splice(i, 1);
    };
    this.dispatchEvent = (event: { type: string }) => {
      for (const fn of listeners[event.type] ?? []) {
        try { fn(event); } catch { /* */ }
      }
    };
  };

  (global as Record<string, unknown>)["Event"] = function Event(
    type: string,
    init?: { bubbles?: boolean; cancelable?: boolean },
  ) {
    this.type = type;
    this.bubbles = init?.bubbles ?? false;
    this.cancelable = init?.cancelable ?? false;
  };

  (global as Record<string, unknown>)["MessageEvent"] = function MessageEvent(
    type: string,
    init?: { data?: unknown },
  ) {
    Event.call(this, type);
    this.data = init?.data ?? null;
  };

  (global as Record<string, unknown>)["ErrorEvent"] = function ErrorEvent(
    type: string,
    init?: { message?: string; error?: unknown },
  ) {
    Event.call(this, type);
    this.message = init?.message ?? "";
    this.error = init?.error ?? null;
  };

  (global as Record<string, unknown>)["CloseEvent"] = function CloseEvent(
    type: string,
    init?: { code?: number; reason?: string; wasClean?: boolean },
  ) {
    Event.call(this, type);
    this.code = init?.code ?? 0;
    this.reason = init?.reason ?? "";
    this.wasClean = init?.wasClean ?? false;
  };

  // ── MessageChannel / MessagePort / BroadcastChannel ──────────────────────

  (global as Record<string, unknown>)["MessageChannel"] = function MessageChannel() {
    const port1Listeners: Array<(e: unknown) => void> = [];
    const port2Listeners: Array<(e: unknown) => void> = [];
    this.port1 = {
      onmessage: null as ((e: unknown) => void) | null,
      postMessage: (data: unknown) => {
        const ev = { data, type: "message" };
        for (const fn of port2Listeners) fn(ev);
        if (this.port2.onmessage) this.port2.onmessage(ev);
      },
      addEventListener: (_: string, fn: (e: unknown) => void) => port1Listeners.push(fn),
    };
    this.port2 = {
      onmessage: null as ((e: unknown) => void) | null,
      postMessage: (data: unknown) => {
        const ev = { data, type: "message" };
        for (const fn of port1Listeners) fn(ev);
        if (this.port1.onmessage) this.port1.onmessage(ev);
      },
      addEventListener: (_: string, fn: (e: unknown) => void) => port2Listeners.push(fn),
    };
  };

  (global as Record<string, unknown>)["BroadcastChannel"] = function BroadcastChannel(
    name: string,
  ) {
    this.name = name;
    this.onmessage = null;
    this.postMessage = (data: unknown) => {
      const r = invoke("broadcast_channel_post", { name, data });
      if (r.error) throw new Error(`BroadcastChannel: ${r.error}`);
    };
    this.close = () => invoke("broadcast_channel_close", { name });
  };

  // ── Worker ────────────────────────────────────────────────────────────────

  (global as Record<string, unknown>)["Worker"] = function Worker(url: string) {
    this.url = url;
    this.onmessage = null;
    this.onerror = null;
    const r = invoke("worker_create", { url });
    this._id = r["id"];
    if (r.error) { this.onerror?.({ message: r.error }); }
    this.postMessage = (data: unknown) =>
      invoke("worker_post_message", { id: this._id, data });
    this.terminate = () => invoke("worker_terminate", { id: this._id });
  };

  // ── ShadowRealm ───────────────────────────────────────────────────────────

  (global as Record<string, unknown>)["ShadowRealm"] = function ShadowRealm() {
    this.evaluate = (code: string) => {
      const r = invoke("shadow_realm_eval", { code });
      if (r.error) throw new Error(`ShadowRealm: ${r.error}`);
      return r["result"];
    };
    this.importValue = (specifier: string, name: string) => {
      const r = invoke("shadow_realm_import", { specifier, name });
      if (r.error) throw new Error(`ShadowRealm: ${r.error}`);
      return r["value"];
    };
  };

  // ── Bun compat ────────────────────────────────────────────────────────────

  const g = global as Record<string, unknown>;
  if (!g["Bun"]) g["Bun"] = {};
  const Bun = g["Bun"] as Record<string, unknown>;

  Bun["serve"] = (opts: { port?: number; hostname?: string }) => {
    const r = invoke("bun_serve", { port: opts.port, hostname: opts.hostname });
    if (r.error) throw new Error(`Bun.serve: ${r.error}`);
    return {
      port: r["port"],
      hostname: r["hostname"],
      stop: () => invoke("bun_serve_stop", { port: r["port"] }),
    };
  };

  Bun["file"] = (path: string) => ({
    path,
    text: () => {
      const r = invoke("bun_file_read", { path, encoding: "utf8" });
      if (r.error) throw new Error(`Bun.file: ${r.error}`);
      return (r["content"] as string) ?? "";
    },
    arrayBuffer: () => {
      const r = invoke("bun_file_read", { path, encoding: "binary" });
      if (r.error) throw new Error(`Bun.file: ${r.error}`);
      return (r["bytes"] as unknown[]) ?? [];
    },
    exists: () => {
      const r = invoke("bun_file_exists", { path });
      return r["exists"] === true;
    },
  });
})(globalThis);
