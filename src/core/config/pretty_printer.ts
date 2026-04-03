// src/core/config/pretty_printer.ts
import { stringify } from "smol-toml";
import type { AlloyConfig } from "./config";

export class PrettyPrinter {
  print(config: AlloyConfig): string {
    return stringify(config as any);
  }
}
