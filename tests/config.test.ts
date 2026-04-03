import { expect, test, describe, beforeAll, afterAll } from "bun:test";
import { ConfigLoader, ConfigValidator, DEFAULT_CONFIG } from "../src/core/config";
import { writeFileSync, unlinkSync, mkdirSync, rmdirSync } from "node:fs";
import { join } from "node:path";

describe("AlloyConfig", () => {
    const testDir = join(process.cwd(), "test-config-dir");
    const localConfigPath = join(testDir, "alloyconfig.toml");

    beforeAll(() => {
        if (!require("node:fs").existsSync(testDir)) mkdirSync(testDir);
    });

    afterAll(() => {
        if (require("node:fs").existsSync(localConfigPath)) unlinkSync(localConfigPath);
        if (require("node:fs").existsSync(testDir)) rmdirSync(testDir);
    });

    test("ConfigLoader: basic loading and defaults", () => {
        const config = ConfigLoader.load(testDir);
        expect(config.target).toBe("local-first");
        expect(config.telemetry).toBe(true);
    });

    test("ConfigLoader: local config overrides defaults", () => {
        writeFileSync(localConfigPath, `
target = "server-first"
telemetry = false
[install]
exact = true
`);
        const config = ConfigLoader.load(testDir);
        expect(config.target).toBe("server-first");
        expect(config.telemetry).toBe(false);
        expect(config.install.exact).toBe(true);
        expect(config.install.dev).toBe(true); // preserved from default
    });

    test("ConfigLoader: environment variable interpolation", () => {
        process.env.TEST_VAR = "hello";
        process.env.TEST_TOKEN = "secret";

        writeFileSync(localConfigPath, `
[define]
API_URL = "https://api.example.com/$TEST_VAR"
TOKEN = "\${TEST_TOKEN}"
`);
        const config = ConfigLoader.load(testDir);
        expect(config.define.API_URL).toBe("https://api.example.com/hello");
        expect(config.define.TOKEN).toBe("secret");
    });

    test("ConfigValidator: validation logic", () => {
        const config = { ...DEFAULT_CONFIG, target: "invalid" as any };
        const { errors } = ConfigValidator.validate(config);
        expect(errors.some(e => e.includes("Invalid target"))).toBe(true);

        const config2 = { ...DEFAULT_CONFIG, target: "local-first" as any, useContainers: false };
        const { errors: errors2 } = ConfigValidator.validate(config2);
        expect(errors2.some(e => e.includes("useContainers cannot be false"))).toBe(true);
    });
});
