import type { BuildPlugin, BuildContext } from "../../../cli/build_plugin";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const SHIM_PATH = join(import.meta.dir, "alloy_async_shim.js");

/**
 * alloy:async BuildPlugin.
 * Injects the async shim into the bundle before the CPS transform.
 */
export const asyncBuildPlugin: BuildPlugin = {
  name: "alloy:async",
  dependsOn: ["alloy:config"],

  before_bundle(source: string, _ctx: BuildContext): string {
    const shim = readFileSync(SHIM_PATH, "utf8");
    return shim + "\n" + source;
  },
};
