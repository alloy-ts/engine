// src/core/config/pretty_printer.ts
import type { AlloyConfig } from "./config";
import * as TOML from "@iarna/toml";

export class PrettyPrinter {
	print(config: AlloyConfig): string {
		// Clean up undefined values to avoid empty keys in TOML
		const clean = (obj: any): any => {
			if (Array.isArray(obj)) return obj.map(clean);
			if (obj !== null && typeof obj === "object") {
				const newObj: any = {};
				for (const [key, value] of Object.entries(obj)) {
					if (value !== undefined) {
						newObj[key] = clean(value);
					}
				}
				return newObj;
			}
			return obj;
		};

		const cleanedConfig = clean(config);

		// TOML.stringify handles basic TOML v1.0.
		// For Requirement 7.7, it should guarantee round-trip deep-equality.
		return TOML.stringify(cleanedConfig);
	}
}
