import { runBeforeBundle, runAfterBundle, type BuildPlugin, type BuildContext } from "./build_plugin";
import { asyncBuildPlugin } from "../core/plugins/async/async_build_plugin";

// Registry of build-time plugins
const BUILD_PLUGINS: BuildPlugin[] = [asyncBuildPlugin];

/**
 * Register a build plugin.
 */
export function registerBuildPlugin(p: BuildPlugin) {
  BUILD_PLUGINS.push(p);
}

/**
 * Orchestrator for the build pipeline.
 * Stages 2 and 3 of the 4-stage pipeline.
 */
export async function runBuildPipeline(source: string, ctx: BuildContext): Promise<string> {
  // Stage 2: before_bundle hooks (shim prepend, JSX transform)
  source = await runBeforeBundle(BUILD_PLUGINS, source, ctx);

  // Stage 3: CPS Transform (to be implemented)
  // source = await runCPSTransform(source, ctx);

  // After-transform hooks
  source = await runAfterBundle(BUILD_PLUGINS, source, ctx);

  return source;
}
