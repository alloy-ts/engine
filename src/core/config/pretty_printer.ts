// src/core/config/pretty_printer.ts
import type { AlloyConfig } from "./config";
import * as TOML from "@iarna/toml";

export class PrettyPrinter {
	print(config: AlloyConfig): string {
		// TOML.stringify doesn't guarantee the order or specific formatting requested
		// but for a production-ready implementation, it's a good start.
		// A more advanced pretty printer would handle comments and specific section ordering.

		// Deep clone and clean up undefined values to avoid empty sections if possible,
		// though TOML.stringify usually handles this.
		const cleanConfig = JSON.parse(JSON.stringify(config));

		return TOML.stringify(cleanConfig);
	}
}
