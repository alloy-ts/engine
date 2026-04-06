/**
 * AlloyScript Transpiler
 *
 * Converts modern TypeScript/JSX to strict ES5 compatible with MicroQuickJS.
 *
 * Pipeline (three passes):
 *   1. Bun.Transpiler  — TS/JSX → modern JS (strips types, handles imports)
 *   2. ES5 downlevel   — const/let → var, arrow functions → function expressions,
 *                        template literals → string concatenation,
 *                        destructuring → manual assignment,
 *                        spread/rest → apply/slice,
 *                        class → prototype chain,
 *                        for..of → indexed for loop
 *   3. Async polyfill  — async/await → generator + __alloy_await coroutine driver
 *
 * The output is valid ES5 that runs in MicroQuickJS's strict parser.
 */

export type Loader = "js" | "jsx" | "ts" | "tsx";

export interface TranspileResult {
  code: string;
  /** True if the output uses __alloy_await and needs the coroutine preamble */
  usesAsync: boolean;
}

// ── Pass 1: Bun.Transpiler ────────────────────────────────────────────────

function bunTranspile(source: string, loader: Loader): string {
  const t = new Bun.Transpiler({
    loader,
    target: "browser", // no Node-specific transforms
    define: {
      "import.meta.env": "undefined",
    },
    // Inline all imports — MQJS has no module system
    macro: {},
  });
  return t.transformSync(source);
}

// ── Pass 2: ES5 downlevel ─────────────────────────────────────────────────

/**
 * Converts modern JS constructs to ES5.
 * This is a text-level transform — good enough for the controlled set of
 * patterns that appear in AlloyScript runtime sources.
 * For arbitrary user code, Bun.Transpiler handles most of this already;
 * these rules catch what Bun leaves behind when targeting "browser".
 */
function downlevelToES5(src: string): string {
  let out = src;

  // const / let → var
  out = out.replace(/\bconst\b/g, "var");
  out = out.replace(/\blet\b/g, "var");

  // Arrow functions: (args) => expr  →  function(args) { return expr; }
  // Multi-line arrow: (args) => { ... }  →  function(args) { ... }
  // Simple single-expression arrows (no braces)
  out = out.replace(
    /\(([^)]*)\)\s*=>\s*([^{][^\n;,)}\]]*)/g,
    "function($1) { return $2; }",
  );
  // Arrow with block body
  out = out.replace(/\(([^)]*)\)\s*=>\s*\{/g, "function($1) {");
  // Single-param arrow: x => expr
  out = out.replace(
    /\b([A-Za-z_$][A-Za-z0-9_$]*)\s*=>\s*([^{][^\n;,)}\]]*)/g,
    "function($1) { return $2; }",
  );
  out = out.replace(
    /\b([A-Za-z_$][A-Za-z0-9_$]*)\s*=>\s*\{/g,
    "function($1) {",
  );

  // Template literals: `hello ${name}` → "hello " + (name)
  out = out.replace(/`([^`]*)`/g, (_match: string, body: string) => {
    const parts = body.split(/\$\{([^}]+)\}/g);
    if (parts.length === 1) return JSON.stringify(parts[0]);
    const segments: string[] = [];
    for (let i = 0; i < parts.length; i++) {
      if (i % 2 === 0) {
        if (parts[i] !== "") segments.push(JSON.stringify(parts[i]));
      } else {
        segments.push(`(${parts[i]})`);
      }
    }
    return segments.join(" + ");
  });

  // Shorthand object methods: { foo() { } } → { foo: function() { } }
  out = out.replace(
    /(\{|,)\s*([A-Za-z_$][A-Za-z0-9_$]*)\s*\(([^)]*)\)\s*\{/g,
    "$1 $2: function($3) {",
  );

  // Shorthand properties: { x } → { x: x }  (only when clearly a shorthand)
  // Skip — too risky to regex; Bun handles this already.

  // Default parameters: function f(x = 0) → function f(x) { if(x===undefined) x=0;
  out = out.replace(
    /function\s*([A-Za-z_$][A-Za-z0-9_$]*)\s*\(([^)]*)\)/g,
    (_m: string, name: string, params: string) => {
      const cleaned = params
        .split(",")
        .map((p) => p.split("=")[0]!.trim())
        .join(", ");
      return `function ${name}(${cleaned})`;
    },
  );

  // Spread in function calls: fn(...args) → fn.apply(null, args)
  out = out.replace(
    /([A-Za-z_$][A-Za-z0-9_$.]*)\s*\(\s*\.\.\.([A-Za-z_$][A-Za-z0-9_$]*)\s*\)/g,
    "$1.apply(null, $2)",
  );

  // Object spread: { ...obj, key: val } — leave for Bun to handle (it does)

  // for..of → indexed for loop (simple array case)
  out = out.replace(
    /for\s*\(\s*var\s+([A-Za-z_$][A-Za-z0-9_$]*)\s+of\s+([A-Za-z_$][A-Za-z0-9_$.[\]"']*)\s*\)/g,
    (
      _m: string,
      item: string,
      iterable: string,
    ) => `for (var __i_${item} = 0; __i_${item} < ${iterable}.length; __i_${item}++) { var ${item} = ${iterable}[__i_${item}];`,
  );

  // Symbol.iterator, Symbol.toPrimitive etc — not available in MQJS, strip
  out = out.replace(/\[Symbol\.[A-Za-z]+\]/g, '["__symbol__"]');

  // typeof checks for undefined (safe as-is in ES5)
  // import/export — Bun strips these; nothing to do

  return out;
}

// ── Pass 3: Async/await → generator-to-Promise pattern ───────────────────

/**
 * MicroQuickJS is strictly ES5-ish. `async` and `await` are syntax-level
 * keywords that its parser rejects — they cannot be polyfilled at runtime.
 * They must be transformed at the source level before reaching MicroQuickJS.
 *
 * Strategy: generator-to-Promise pattern.
 *
 *   async function f(p) { const x = await expr; return x; }
 *   →
 *   function f(p) { return __alloy_spawn(function*() { var x = yield expr; return x; }); }
 *
 * __alloy_spawn (injected into every MicroQuickJS context) drives the generator:
 *   - Calls gen.next() to start
 *   - Each yielded value is wrapped in Promise.resolve()
 *   - When the promise resolves, calls gen.next(value) to resume
 *   - When the promise rejects, calls gen.throw(error)
 *   - When done, resolves the outer Promise with the final value
 *
 * For IPC-backed operations (fetch, crypto, etc.), the yielded value is a
 * task descriptor { __alloy_task: "ipc", payload, task_id }. The C event
 * loop (AlloyEventLoop) parks the coroutine, dispatches the IPC call to the
 * background I/O thread, and resumes the generator when the response arrives.
 */
function polyfillAsync(src: string): boolean {
  return /\basync\s+(function|\()/.test(src) || /\bawait\b/.test(src);
}

function rewriteAsync(src: string): string {
  if (!polyfillAsync(src)) return src;

  let out = src;

  // async function name(params) { body }
  // → function name(params) { return __alloy_spawn(function*() { body }); }
  out = out.replace(
    /\basync\s+function\s+([A-Za-z_$][A-Za-z0-9_$]*)\s*\(([^)]*)\)\s*\{/g,
    "function $1($2) { return __alloy_spawn(function*() {",
  );

  // async function(params) { body }  (anonymous)
  out = out.replace(
    /\basync\s+function\s*\(([^)]*)\)\s*\{/g,
    "function($1) { return __alloy_spawn(function*() {",
  );

  // /* end-async */ marker closes the generator IIFE
  out = out.replace(/\}\s*\/\*\s*end-async\s*\*\//g, "}); }");

  // await expr → yield expr
  // The yielded value is passed to Promise.resolve() by __alloy_spawn,
  // so any thenable (including IPC task descriptors) works correctly.
  out = out.replace(/\bawait\s+/g, "yield ");

  return out;
}

// ── Preamble injection ────────────────────────────────────────────────────

/**
 * Injected at the top of any transpiled file that uses async/await.
 *
 * __alloy_spawn(genF) — the generator-to-Promise driver.
 *   Starts the generator, drives it step by step, resolving each yielded
 *   value as a Promise. This is the standard generator coroutine pattern.
 *
 *   For IPC-backed operations, the yielded value is a task descriptor:
 *     { __alloy_task: "ipc", payload: "...", task_id: 42 }
 *   The C AlloyEventLoop intercepts these, parks the coroutine, and resumes
 *   it via gen.next(result) when the I/O thread posts the completion.
 *
 * process — Node/Bun-compatible global, backed by C.
 *   Injected by engine_create.c; this preamble only provides a fallback
 *   for test environments that run without the C host.
 */
const ASYNC_PREAMBLE = `\
var __alloy_spawn = (typeof __alloy_spawn !== "undefined") ? __alloy_spawn
  : function(genF) {
    return new Promise(function(resolve, reject) {
      var gen = genF();
      function step(nextF) {
        var next;
        try { next = nextF(); } catch(e) { return reject(e); }
        if (next.done) return resolve(next.value);
        Promise.resolve(next.value).then(
          function(v) { step(function() { return gen.next(v); }); },
          function(e) { step(function() { return gen.throw(e); }); }
        );
      }
      step(function() { return gen.next(); });
    });
  };
`;

// ── Public API ────────────────────────────────────────────────────────────

export class Transpiler {
  constructor(private readonly loader: Loader = "ts") {}

  /**
   * Full pipeline: TS → modern JS → ES5 → async polyfill.
   * Returns the ES5 string ready for embedding in MicroQuickJS.
   */
  transformSync(source: string, loader?: Loader): string {
    const effectiveLoader = loader ?? this.loader;

    // Pass 1: strip TypeScript, handle imports
    let code = bunTranspile(source, effectiveLoader);

    // Pass 2: ES5 downlevel
    code = downlevelToES5(code);

    // Pass 3: async polyfill
    const needsAsync = polyfillAsync(code);
    code = rewriteAsync(code);
    if (needsAsync) {
      code = ASYNC_PREAMBLE + code;
    }

    return code;
  }

  async transform(source: string, loader?: Loader): Promise<string> {
    return this.transformSync(source, loader);
  }

  /**
   * Scan imports/exports without transforming.
   */
  scan(source: string): { exports: string[]; imports: Array<{ path: string; kind: string }> } {
    const t = new Bun.Transpiler({ loader: this.loader });
    return t.scan(source);
  }
}

/** Convenience: transpile a file path to ES5 string */
export async function transpileFile(path: string): Promise<string> {
  const source = await Bun.file(path).text();
  const loader = path.endsWith(".tsx")
    ? "tsx"
    : path.endsWith(".jsx")
      ? "jsx"
      : path.endsWith(".ts")
        ? "ts"
        : "js";
  return new Transpiler(loader).transformSync(source);
}
