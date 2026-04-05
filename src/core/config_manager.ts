import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import { parse, stringify } from "smol-toml";

export interface AlloyPlugin {
    name: string;
    onLoad?: (config: any) => void;
    hooks?: Record<string, Function>;
}

export class ConfigManager {
    private config: any = {};
    private configPath: string;

    constructor(projectRoot: string) {
        this.configPath = join(projectRoot, "alloyconfig.toml");
        this.load();
    }

    private load() {
        if (existsSync(this.configPath)) {
            const content = readFileSync(this.configPath, "utf-8");
            this.config = parse(content);
        }
    }

    public get(key: string, defaultValue?: any): any {
        return this.config[key] ?? defaultValue;
    }

    public set(key: string, value: any) {
        this.config[key] = value;
        const content = stringify(this.config);
        writeFileSync(this.configPath, content);
    }

    public registerPlugin(plugin: AlloyPlugin) {
        if (plugin.onLoad) {
            plugin.onLoad(this.get(plugin.name, {}));
        }
    }
}
