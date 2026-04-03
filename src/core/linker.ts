import { type Loader, Transpiler } from "./transpiler";
import * as path from "node:path";
import * as fs from "node:fs";

export interface LinkerOptions {
	minify?: boolean;
	treeShaking?: boolean;
}

export class Linker {
	private transpiler: Transpiler;

	constructor() {
		this.transpiler = new Transpiler();
	}

	/**
	 * Performs dead code elimination and tree-shaking on JS source.
	 * Currently a simplified version that BFS traverses imports.
	 */
	async link(entrypoints: string[], options: LinkerOptions = {}): Promise<string> {
		const visited = new Set<string>();
		const queue = [...entrypoints];
		let bundle = "";

		while (queue.length > 0) {
			const curr = queue.shift()!;
			if (visited.has(curr)) continue;
			visited.add(curr);

			const content = fs.readFileSync(curr, "utf-8");
			const loader = path.extname(curr).slice(1) as Loader;

			// Scan imports for BFS
			const { imports } = this.transpiler.scan(content);
			for (const imp of imports) {
				if (imp.kind === "import-statement" || imp.kind === "require-call") {
					// Simplified resolution
					const resolved = path.resolve(path.dirname(curr), imp.path);
					const ext = [".ts", ".tsx", ".js", ".jsx"].find(e => fs.existsSync(resolved + e)) || "";
					if (ext) queue.push(resolved + ext);
				}
			}

			// Transpile and append to bundle
			const transformed = this.transpiler.transformSync(content, loader);
			bundle += `\n/* module: ${curr} */\n` + transformed;
		}

		if (options.treeShaking) {
			// Tree-shaking logic would go here:
			// 1. Parse bundle with acorn
			// 2. Mark & Sweep reachable nodes starting from entry point exports
			// 3. Remove unreachable nodes
			// For now, this BFS traversal serves as a coarse-grained tree-shaking (only including imported files)
		}

		return bundle;
	}
}
