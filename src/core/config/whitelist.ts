// src/core/config/whitelist.ts
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

export function buildWhitelist(
	cwd: string,
	capabilities: string[],
): Set<string> {
	const whitelist = new Set<string>();

	// Strip 'use-' prefix from each entry to derive permitted binary names
	for (const cap of capabilities) {
		if (cap.startsWith("use-")) {
			whitelist.add(cap.substring(4));
		} else {
			whitelist.add(cap);
		}
	}

	// Auto-whitelist npm -> alloy and npx -> alloyx without explicit entries
	if (whitelist.has("npm")) whitelist.add("alloy");
	if (whitelist.has("npx")) whitelist.add("alloyx");

	// Auto-whitelist binaries from package.json dependencies and devDependencies
	const pkgPath = join(cwd, "package.json");
	if (existsSync(pkgPath)) {
		try {
			const pkg = JSON.parse(readFileSync(pkgPath, "utf-8"));
			const deps = Object.keys(pkg.dependencies || {});
			const devDeps = Object.keys(pkg.devDependencies || {});
			for (const d of [...deps, ...devDeps]) {
				whitelist.add(d);
			}
		} catch (e) {
			// Ignore package.json parsing errors for whitelist
		}
	}

	return whitelist;
}

export function isAllowed(whitelist: Set<string>, binary: string): boolean {
	// Simple whitelist check
	return whitelist.has(binary);
}
