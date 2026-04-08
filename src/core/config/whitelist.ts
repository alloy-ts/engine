// src/core/config/whitelist.ts
import { readFileSync, existsSync } from "node:fs";
import { join } from "node:path";

export interface Whitelist {
	isAllowed(binary: string): boolean;
}

const AUTO_WHITELISTED = ["npm", "npx"];

export function buildWhitelist(
	capabilities: string[],
	cwd: string = process.cwd(),
): Whitelist {
	const allowed = new Set<string>(AUTO_WHITELISTED);

	// Strip "use-" prefix
	for (const cap of capabilities) {
		if (cap.startsWith("use-")) {
			allowed.add(cap.substring(4));
		} else {
			allowed.add(cap);
		}
	}

	// Auto-whitelist binaries from package.json dependencies
	const packageJsonPath = join(cwd, "package.json");
	if (existsSync(packageJsonPath)) {
		try {
			const packageJson = JSON.parse(readFileSync(packageJsonPath, "utf-8"));
			const deps = {
				...packageJson.dependencies,
				...packageJson.devDependencies,
			};
			for (const dep of Object.keys(deps)) {
				allowed.add(dep);
			}
		} catch (e) {
			// Ignore malformed package.json
		}
	}

	return {
		isAllowed(binary: string) {
			// npm -> alloy, npx -> alloyx
			if (binary === "npm" || binary === "npx") return true;
			return allowed.has(binary);
		},
	};
}
