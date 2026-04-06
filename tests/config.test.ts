import { describe, it, expect, beforeEach, afterEach } from "bun:test";
import { ConfigLoader } from "../src/core/config/loader";
import { ConfigValidator } from "../src/core/config/validator";
import { mkdirSync, writeFileSync, rmSync } from "node:fs";
import { join } from "node:path";

describe("AlloyConfig", () => {
    const testDir = join(process.cwd(), "temp-test-config");

    beforeEach(() => {
        if (mkdirSync) mkdirSync(testDir, { recursive: true });
    });

    afterEach(() => {
        rmSync(testDir, { recursive: true, force: true });
    });

    describe("ConfigLoader", () => {
        it("basic loading and defaults", () => {
            const loader = new ConfigLoader();
            const config = loader.load({ cwd: testDir });
            expect(config.target).toBe("local-first");
            expect(config.telemetry).toBe(true);
            expect(config.test?.root).toBe(".");
        });

        it("local config overrides defaults", () => {
            const configPath = join(testDir, "alloyconfig.toml");
            writeFileSync(configPath, 'target = "server-first"\ntelemetry = false\n[test]\nroot = "./src"');
            const loader = new ConfigLoader();
            const config = loader.load({ cwd: testDir });
            expect(config.target).toBe("server-first");
            expect(config.telemetry).toBe(false);
            expect(config.test?.root).toBe("./src");
            // Defaults should still be injected correctly for target:
            expect(config.disableWasmEdge).toBe(true);
            expect(config.useContainers).toBe(false);
        });

        it("environment variable interpolation", () => {
            process.env.TEST_VAR = "hello-alloy";
            const configPath = join(testDir, "alloyconfig.toml");
            writeFileSync(configPath, 'preload = ["$TEST_VAR"]\n[define]\nAPP_VERSION = "${TEST_VAR}"');
            const loader = new ConfigLoader();
            const config = loader.load({ cwd: testDir });
            expect(config.preload).toContain("hello-alloy");
            expect(config.define?.APP_VERSION).toBe("hello-alloy");
            delete process.env.TEST_VAR;
        });
    });

    describe("ConfigValidator", () => {
        it("validation logic", () => {
            const validator = new ConfigValidator();

            // Invalid target
            const res1 = validator.validate({ target: "invalid" }, "test.toml");
            expect(res1.valid).toBe(false);
            expect(res1.errors.some(e => e.fieldPath === "target")).toBe(true);

            // Cross-field error: useContainers=false with local-first
            const res2 = validator.validate({ target: "local-first", useContainers: false }, "test.toml");
            expect(res2.valid).toBe(false);
            expect(res2.errors.some(e => e.fieldPath === "useContainers")).toBe(true);

            // Cross-field warning: seed without randomize
            const res3 = validator.validate({ test: { seed: 123, randomize: false } }, "test.toml");
            expect(res3.warnings.some(w => w.fieldPath === "test.seed")).toBe(true);
        });
    });
});
