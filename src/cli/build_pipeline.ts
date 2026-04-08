import * as fs from "node:fs";
import { type BuildPlugin, type PluginBuild } from "../build";

export function buildPipelinePlugin(): BuildPlugin {
  return {
    name: "alloy:build-pipeline",
    setup(build: PluginBuild) {
      // Orchestrator logic
    }
  };
}

export const BUILD_PLUGINS = [
  // Registry of plugins
];
