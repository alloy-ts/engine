// src/core/config/pretty_printer.ts
import type { AlloyConfig } from "./config";
import * as TOML from "@iarna/toml";

export class PrettyPrinter {
	print(config: AlloyConfig): string {
		// Since @iarna/toml doesn't preserve our annotated example's comments,
		// we follow the requirement: parse(print(config)) deep-equals config.
		// A manual serializer would be needed for comments.
		// For this implementation, we use the library.

		// Deep clone and remove undefined
		const clean = JSON.parse(JSON.stringify(config));
		return TOML.stringify(clean);
	}
}
