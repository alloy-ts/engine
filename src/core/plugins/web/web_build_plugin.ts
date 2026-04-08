// src/core/plugins/web/web_build_plugin.ts

export interface BuildPlugin {
    name: string;
    dependsOn?: string[];
    before_bundle?: (source: string, ctx: any) => string;
}

export const webBuildPlugin: BuildPlugin = {
    name: "alloy:web",
    dependsOn: ["alloy:config"],
    before_bundle: (source, ctx) => {
        const shimHeader = `// Alloy Web Shims\n`;
        return shimHeader + source;
    }
};
