import TOML from "@iarna/toml";
import type { AlloyConfig } from "./schema";

export class PrettyPrinter {
	print(config: AlloyConfig): string {
		return TOML.stringify(config as any);
	}
}
