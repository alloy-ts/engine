// src/core/config/pretty_printer.ts
import type { AlloyConfig } from "./config";
import * as TOML from "@iarna/toml";

export class PrettyPrinter {
	print(config: AlloyConfig): string {
		// Cleanup config before stringifying to ensure only valid fields are present
		const clean = JSON.parse(JSON.stringify(config));

		// Ensure we don't have circular references (already handled by stringify)
		// and remove any potential undefined values.

		return TOML.stringify(clean);
	}
}
