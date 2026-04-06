import * as TOML from "smol-toml";
import type { AlloyConfig } from "./config";

export class PrettyPrinter {
  print(config: AlloyConfig): string {
    return TOML.stringify(config);
  }
}
