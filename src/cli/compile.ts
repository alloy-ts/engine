import { build } from "../core/build";

export async function compile(entrypoint: string, options: any) {
  // Stage 1: esbuild TS strip + bundle
  // Stage 2: runBeforeBundle with BuildPlugins
  // Stage 3: CPS transform
  // Stage 4: alloyc_compile
  console.log(`[cli] Compiling ${entrypoint}...`);
  const result = await build({
    entrypoints: [entrypoint],
    outdir: options.outdir || "./dist"
  });
  return result;
}
