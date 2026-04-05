import { ConfigManager } from "./config_manager";
import { Transpiler } from "./transpiler";
import { ensureAlloyHost } from "./host_manager";
import { join } from "node:path";
import { existsSync, mkdirSync, writeFileSync } from "node:fs";

async function testIntegration() {
    console.log("Starting AlloyScript Integration Test...");
    const projectRoot = join(process.cwd(), "test-project");
    if (!existsSync(projectRoot)) mkdirSync(projectRoot);

    // 1. Config Manager
    const configManager = new ConfigManager(projectRoot);
    configManager.set("test-plugin", { enabled: true });
    console.log("Config Manager check:", configManager.get("test-plugin"));

    // 2. Transpiler
    const transpiler = new Transpiler();
    const code = "async function hello() { await console.log('hello'); }";
    const transformed = transpiler.transformSync(code);
    console.log("Transpiler check (async/await polyfill):", transformed.includes("yield"));

    // 3. Host Manager (Simulated)
    try {
        const hostPath = await ensureAlloyHost();
        console.log("Host Manager check (binary path):", hostPath);
    } catch (e) {
        console.warn("Host Manager check (expected error in mock env):", (e as Error).message);
    }

    console.log("Integration Test Complete.");
}

testIntegration();
