/**
 * AlloyScript — BuildPlugin interface
 *
 * Build-time counterpart to RuntimePlugin (include/alloy/plugin.h).
 * Lives entirely in the Bun CLI pipeline — never loaded by alloy_host.
 *
 * Hook execution order in alloyc:
 *
 *   before_bundle(source, ctx)   — called after esbuild TS strip, before
 *                                  any further transforms. Use to prepend
 *                                  shims or rewrite source (e.g. JSX → calls).
 *                                  Plugins run in registration order; each
 *                                  receives the output of the previous.
 *
 *   after_bundle(source, ctx)    — called after all before_bundle hooks and
 *                                  the CPS transform. Use for post-transform
 *                                  validation or source map fixups.
 *
 * Plugin registration:
 *   Import and push into the BUILD_PLUGINS array in src/cli/build_pipeline.ts.
 *   Order matters: before_bundle hooks run in array order.
 *
 * Current plugins:
 *   alloy:async  →  before_bundle: prepend alloy_async_shim.js
 *   alloy:gui    →  before_bundle: JSX → alloy.ui.* calls
 */

export interface BuildContext {
  /** Logical filename being compiled (for error messages). */
  filename: string;
  /** Build options passed to alloyc. */
  debug: boolean;
  target: "desktop" | "mobile" | "server";
}

export interface BuildPlugin {
  /** Plugin identifier, e.g. "alloy:async", "alloy:gui". */
  name: string;

  /**
   * before_bundle — transform source after esbuild TS strip, before CPS pass.
   *
   * Return the transformed source string, or the original source unchanged.
   * Throw to abort the build with an error.
   */
  before_bundle?(source: string, ctx: BuildContext): string | Promise<string>;

  /**
   * after_bundle — inspect or patch source after all transforms complete.
   *
   * Return the (optionally modified) source string.
   * Throw to abort the build with an error.
   */
  after_bundle?(source: string, ctx: BuildContext): string | Promise<string>;
}

/**
 * Run all registered plugins' before_bundle hooks in order.
 * Each plugin receives the output of the previous.
 */
export async function runBeforeBundle(
  plugins: BuildPlugin[],
  source: string,
  ctx: BuildContext,
): Promise<string> {
  let out = source;
  for (const plugin of plugins) {
    if (plugin.before_bundle) {
      out = await plugin.before_bundle(out, ctx);
    }
  }
  return out;
}

/**
 * Run all registered plugins' after_bundle hooks in order.
 */
export async function runAfterBundle(
  plugins: BuildPlugin[],
  source: string,
  ctx: BuildContext,
): Promise<string> {
  let out = source;
  for (const plugin of plugins) {
    if (plugin.after_bundle) {
      out = await plugin.after_bundle(out, ctx);
    }
  }
  return out;
}
