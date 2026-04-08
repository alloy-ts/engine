import { useConfig } from "../../core/config/config";

/**
 * Register configuration for the alloy:async plugin.
 */
export const asyncConfig = useConfig({
  configs: {
    dispatch_queue_cap: { default: 64 },
    log_unhandled_rejections: { default: true },
    shim_override_path: { default: "" },
  }
}, { name: "alloy:async" });
