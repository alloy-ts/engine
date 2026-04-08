import { Alloy } from "../../runtime";
import { useConfig as useAlloyConfig } from "../config/use_config";

export function setupAsyncConfig() {
    return useAlloyConfig({
        configs: {
            "dispatch_queue_cap": { default: 64 },
            "log_unhandled_rejections": { default: true },
            "shim_override_path": { default: "" }
        }
    });
}
