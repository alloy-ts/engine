// src/core/plugins/async/async_build_plugin.ts
import type { BuildPlugin } from "../../plugin";
import { readFileSync } from "node:fs";
import { join } from "node:path";

export const asyncBuildPlugin: BuildPlugin = {
    name: "alloy:async",
    // before_bundle hook implementation would go here
};
