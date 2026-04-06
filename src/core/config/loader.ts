import { readFile } from "node:fs/promises";
import { join, dirname } from "node:path";
import { AlloyConfigSchema, type AlloyConfig } from "./schema";
import * as TOML from "smol-toml";

export class ConfigLoader {
    async load(options: { cwd?: string, configPath?: string, cliFlags?: Partial<AlloyConfig> } = {}): Promise<AlloyConfig> {
        const cwd = options.cwd || process.cwd();
        const path = options.configPath || join(cwd, "alloyconfig.toml");

        let raw = {};
        try {
            const content = await readFile(path, "utf-8");
            raw = TOML.parse(content);
        } catch (e) {}

        /* Requirement 1.3: apply precedence order */
        const merged = { ...raw, ...options.cliFlags };

        /* Requirement 8.1: Env var interpolation */
        const interpolated = this.interpolate(merged);

        return AlloyConfigSchema.parse(interpolated);
    }

    private interpolate(obj: any): any {
        if (typeof obj === "string") {
            return obj.replace(/\$(\w+)|\${(\w+)}/g, (_, v1, v2) => {
                const name = v1 || v2;
                return process.env[name] || "";
            });
        }
        if (Array.isArray(obj)) return obj.map(x => this.interpolate(x));
        if (typeof obj === "object" && obj !== null) {
            const res: any = {};
            for (const key in obj) res[key] = this.interpolate(obj[key]);
            return res;
        }
        return obj;
    }
}
