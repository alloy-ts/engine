export class Whitelist {
	private allowedBinaries: Set<string>;

	constructor(capabilities: string[], extraBinaries: string[] = []) {
		this.allowedBinaries = new Set<string>();

		// 1. Process shellCapabilities (strip 'use-' prefix)
		for (const cap of capabilities) {
			if (cap.startsWith("use-")) {
				this.allowedBinaries.add(cap.substring(4));
			} else {
				this.allowedBinaries.add(cap);
			}
		}

		// 2. Auto-whitelist npm and npx
		this.allowedBinaries.add("npm");
		this.allowedBinaries.add("npx");

		// 3. Add extra binaries (from package.json dependencies, etc.)
		for (const bin of extraBinaries) {
			this.allowedBinaries.add(bin);
		}
	}

	isAllowed(binary: string): boolean {
		// Translation: npm -> alloy, npx -> alloyx (handled by runtime, but whitelist allows the names)
		if (binary === "alloy" || binary === "alloyx") return true;
		return this.allowedBinaries.has(binary);
	}
}

export function buildWhitelist(
	capabilities: string[],
	packageJson?: any,
): Whitelist {
	const extraBinaries: string[] = [];
	if (packageJson) {
		const deps = {
			...(packageJson.dependencies || {}),
			...(packageJson.devDependencies || {}),
		};
		// For simplicity in this implementation, we treat dependency names as binary names
		// In a full implementation, we might look into node_modules/.bin
		for (const dep of Object.keys(deps)) {
			extraBinaries.push(dep);
		}
	}
	return new Whitelist(capabilities, extraBinaries);
}
