import * as fs from "node:fs";
import { type PluginBuild } from "../build";

export function asyncBuildPlugin() {
  return {
    name: "alloy:async",
    setup(build: PluginBuild) {
      build.onLoad({ filter: /.*/ }, async (args) => {
        // Prepend async shim for ES5 compatibility
        const shim = fs.readFileSync("src/core/plugins/async/alloy_async_shim.js", "utf-8");
        const content = fs.readFileSync(args.path, "utf-8");
        return {
          contents: shim + "\n" + content,
          loader: "js"
        };
      });
    }
  };
}
