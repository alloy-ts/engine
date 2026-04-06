// src/core/config/whitelist.ts
import { readFileSync, existsSync } from "node:fs";
import { join } from "node:path";

export function buildWhitelist(
	capabilities: string[],
	cwd: string = process.cwd(),
): { isAllowed: (binary: string) => boolean } {
	const allowed = new Set<string>();

	// Strip 'use-' prefix
	for (const cap of capabilities) {
		if (cap.startsWith("use-")) {
			allowed.add(cap.substring(4));
		} else {
			allowed.add(cap);
		}
	}

	// Auto-whitelist npm -> alloy, npx -> alloyx
	allowed.add("alloy");
	allowed.add("alloyx");
	// Original npm/npx should be checked too, though they might be replaced
	allowed.add("npm");
	allowed.add("npx");

	// Auto-whitelist from package.json
	const pkgPath = join(cwd, "package.json");
	if (existsSync(pkgPath)) {
		try {
			const pkg = JSON.parse(readFileSync(pkgPath, "utf-8"));
			const deps = { ...pkg.dependencies, ...pkg.devDependencies };
			for (const dep in deps) {
				// Simplified: assume dependency name can be a binary name
				// In practice, this would look at bin field of the package
				allowed.add(dep);
			}
		} catch (e) {}
	}

	return {
		isAllowed: (binary: string) => allowed.has(binary),
	};
}
