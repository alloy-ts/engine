import { readFileSync, existsSync } from "node:fs";
import { join } from "node:path";

export function buildWhitelist(
	capabilities: string[],
	cwd: string = process.cwd(),
): string[] {
	const whitelist = new Set<string>();

	// 1. Strip use- prefix
	for (const cap of capabilities) {
		if (cap.startsWith("use-")) {
			whitelist.add(cap.substring(4));
		}
	}

	// 2. Auto-whitelist npm -> alloy and npx -> alloyx
	whitelist.add("alloy");
	whitelist.add("alloyx");

	// 3. Auto-whitelist binaries from package.json
	const pkgPath = join(cwd, "package.json");
	if (existsSync(pkgPath)) {
		try {
			const pkg = JSON.parse(readFileSync(pkgPath, "utf-8"));
			const deps = { ...pkg.dependencies, ...pkg.devDependencies };
			for (const dep of Object.keys(deps)) {
				// Simple heuristic: many packages provide a binary with their name
				whitelist.add(dep);
			}
		} catch (e) {}
	}

	return Array.from(whitelist);
}
