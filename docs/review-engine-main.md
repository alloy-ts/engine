# Engine review — `main` @ `bdb618d`

> Reviewer: automated review pass. Head reviewed: `origin/main` tip `bdb618d`
> ("docs: describe completion records, environments and declarations").
> Branch this review lives on: `review/engine-main`.

## Verdict

`main` is a clean, well-tested **MVP subset** of an ECMAScript runtime, and it
builds and tests green. The architecture is sound (pure Rust, no `unsafe`, host
boundary fully isolated in the napi facade). The main gaps are scope, not
quality: the interpreter is **expression-only** today, and the commit narrative
is ahead of the code in places.

## Build & test status — all green

| Check | Result |
|---|---|
| `engine`: `cargo check` | ✅ pass |
| `engine`: `cargo test` | ✅ 27/27 unit tests pass (lexer, parser, evaluator, context, heap, number, realm) |
| `core` (napi facade): `cargo check` | ✅ pass — compiles against pinned `alloy-engine@b0476aa` |

Notes:
- `engine` main has **no napi/Node layer** by design (commit `b0476aa`
  "extract alloy-engine as a standalone Rust crate"). The TS tests from the old
  `value-layer` work (`host_globals.test.ts`, `number_semantics.test.ts`) lived
  in the previous napi tree and are **not present in engine main**; they now
  belong to `core`. `core`'s `npm test` (node + napi build) was **not** executed
  here — only the Rust `cargo check` was.

## Exported `Engine` API (built from **core**, not engine)

`engine` is a pure Rust library and produces **no `dist`**. The JS-facing
`Engine` class is the napi facade in `alloy-ts/core` (`src/engine.rs`), built to
`dist/index.mjs` (package name is the misnomer `"repository"`). Confirmed surface:

```ts
const engine = new Engine();            // Engine
const ctx = engine.createContext();     // -> Context (own Realm + heap)
ctx.eval(source: string): undefined | null | boolean | number | string
ctx.get(name: string):  undefined | null | boolean | number | string
ctx.set(name: string, value: undefined|null|boolean|number|string): void
ctx.dispose(): void
ctx.disposed: boolean                    // getter
```

Errors surface as host throws of the captured `SyntaxError`/`ReferenceError`/
`TypeError`/`RangeError` constructors (patched globals are ignored). Return type
is the 5 MVP primitives only.

## What's implemented well

- **Clean value model.** `Value` is `Undefined | Null | Boolean | Number(f64) |
  String(StringHandle)`; strings are heap handles scoped to a `Realm`. Deliberately
  decoupled from the host `napi_value` and from core's wrapper types.
- **Sound lifecycle.** `RuntimeContext` wraps `Realm` with explicit `dispose()`;
  post-dispose ops fail with `Disposed`. Contexts are isolated (verified by test).
- **Spec-faithful primitives.** `number.rs` does `Number::toString` / string-to-
  number to the spec layout; evaluator tests cover `-0`, `NaN`, `Infinity`,
  remainder sign, and strict/loose equality (incl. `0 == -0`, `null == undefined`).
- **Completion records** exist (`Normal`/`Throw`, `Exception`, `ErrorKind`);
  error constructors map to host types.

## What's missing (MVP gaps)

1. **Statements / declarations — none.** `script.rs`'s `Expr` has only
   `Number | String | Boolean | Null | Identifier | Unary | Binary`;
   `ScriptRecord` is `Vec<Expr>`. There is **no** `var`/`let`/`const`, no
   assignment (`x = 1` → `SyntaxError`), no blocks, no `if`/`while`/`for`,
   no `return`. `parser.rs:103` explicitly rejects call expressions
   (`"Call expressions are not supported"`).
2. **No objects / functions.** `Value` has no `Object` or `Function` variant, so
   no property access/mutation, no prototypes, no `this`, no function literals or
   calls.
3. **No `Symbol` / `BigInt`.** Numbers are `f64` only; the integer/bigint wrappers
   from the previous napi tree are not in `engine` main.
4. **Abrupt completions limited.** Only `Normal`/`Throw`; no `break`/`continue`/
   `return`, which is moot until loops/functions exist.
5. **No scoping chain.** Only a single global environment per `Realm`; block/
   function environments are not wired into the parser/evaluator despite the
   "environment records" commit message.

## Discrepancy: commit narrative vs. tree

`main`'s history advertises `33b3339 "environment records, var/let/const,
assignment and blocks"` and `1b03642 "completion records with UpdateEmpty"`, but
the current AST and evaluator implement **neither** var/let/const/assignment/blocks
nor any extra completion variants. Either those changes were reverted/refactored
out, or the commit messages describe a plan not reflected in the tree. Recommend
aligning commit/PR descriptions with the actual AST, or landing the advertised
features — otherwise reviewers will expect capabilities that don't parse.

## `examples/engine.ts` — feasible? Yes (basic smoke only)

Enough exists to draft a quick example, **with the caveat** that only primitive
expressions and globals work — no assignment, declarations, calls, or objects.

```ts
// examples/engine.ts — smoke test of the Engine facade (built from alloy-ts/core)
// Run with: node --import ./dist/index.js examples/engine.ts
import { Engine, Context } from "repository"; // core's package name is "repository"

const engine = new Engine();
const ctx: Context = engine.createContext();

ctx.set("x", 21);
ctx.set("name", "Ada");
console.log(ctx.eval("x * 2"));          // 42
console.log(ctx.eval('"hi " + name'));   // "hi Ada"
console.log(ctx.eval("1 / 0"));          // Infinity
console.log(ctx.eval("NaN === NaN"));    // false

try {
  ctx.eval("missing");                   // ReferenceError
} catch (e) {
  console.log((e as Error).name);        // "ReferenceError"
}

ctx.dispose();
console.log(ctx.disposed);               // true
```

Do **not** add `const y = 1`, `y = 1`, `foo()`, or `{...}` to the example — they
are `SyntaxError` today.

## Open questions / next steps

- Is the expression-only state the intended MVP, or should `33b3339`/`1b03642`'s
  advertised features be (re)landed before this is called "engine"?
- Where should `examples/engine.ts` live? `engine` has no `package.json`/`dist`,
  so it must live with `core` (the facade that produces `dist/index.mjs`).
- Carry-over note for any re-import of the old `value-layer` hardening work into
  this pure-Rust engine: keep the captured-intrinsics discipline and `SAFETY`
  comments; if a `Number::toString` analogue returns here, capture the host
  `Number.prototype.toString` at init rather than re-reading it per call.
