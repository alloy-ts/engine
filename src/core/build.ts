import { Transpiler } from "./transpiler";
import { writeFileSync, readFileSync } from "node:fs";
import { join } from "node:path";

export async function build(options: any) {
    const outputs = [];
    for (const entry of options.entrypoints) {
        const code = readFileSync(entry, "utf-8");
        const transpiler = new Transpiler(options);
        const transformed = transpiler.transformSync(code);

        const outPath = join(options.outdir || "./dist", entry.replace(/\.ts$/, ".js"));
        // mkdirp outdir...
        writeFileSync(outPath, transformed);
        outputs.push({ path: outPath, kind: "entry-point", size: transformed.length });
    }
    return { success: true, outputs, logs: [] };
}
