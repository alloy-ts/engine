// src/core/config/__tests__/config_test.ts
import { expect, test, describe, afterEach, beforeEach } from "bun:test";
import { ConfigLoader } from "../loader";
import { ConfigValidator } from "../validator";
import { join } from "node:path";
import { writeFileSync, unlinkSync, mkdirSync, existsSync } from "node:fs";

describe("ConfigLoader", () => {
  const tmpDir = join(process.cwd(), "tmp-test-config");

  beforeEach(() => {
    if (!existsSync(tmpDir)) {
      mkdirSync(tmpDir);
    }
  });

  test("discovery and merge", () => {
    const loader = new ConfigLoader();
    const workspaceConfig = join(tmpDir, "alloyconfig.toml");
    writeFileSync(workspaceConfig, `
target = "server-first"
[test]
root = "./tests"
    `);

    const config = loader.load({ cwd: tmpDir, configPath: "alloyconfig.toml" });
    expect(config.target).toBe("server-first");
    expect(config.test?.root).toBe("./tests");
    expect(config.disableWasmEdge).toBe(true); // Default for server-first

    unlinkSync(workspaceConfig);
  });

  test("interpolation", () => {
    const loader = new ConfigLoader();
    process.env.TEST_VAR = "hello";
    const workspaceConfig = join(tmpDir, "alloyconfig.toml");
    writeFileSync(workspaceConfig, `
jsxImportSource = "preact-$TEST_VAR"
    `);

    const config = loader.load({ cwd: tmpDir, configPath: "alloyconfig.toml" });
    expect(config.jsxImportSource).toBe("preact-hello");

    delete process.env.TEST_VAR;
    unlinkSync(workspaceConfig);
  });

  test("define merge", () => {
    const loader = new ConfigLoader();
    const workspaceConfig = join(tmpDir, "alloyconfig.toml");
    writeFileSync(workspaceConfig, `
[define]
A = "1"
    `);

    const config = loader.load({
        cwd: tmpDir,
        configPath: "alloyconfig.toml",
        cliFlags: { define: { B: "2" } }
    });
    expect(config.define?.A).toBe("1");
    expect(config.define?.B).toBe("2");

    unlinkSync(workspaceConfig);
  });

  test("requirement 1.9: alloy run ignores user config", () => {
    const loader = new ConfigLoader();
    const userConfig = join(tmpDir, ".alloyconfig.toml");
    process.env.HOME = tmpDir;
    writeFileSync(userConfig, `
logLevel = "debug"
    `);

    // Default load (not run) should include user config
    const config1 = loader.load({ cwd: tmpDir });
    expect(config1.logLevel).toBe("debug");

    // Load for "run" command
    const config2 = loader.load({ cwd: tmpDir, command: "run" });
    expect(config2.logLevel).toBe("warn"); // Default

    unlinkSync(userConfig);
    delete process.env.HOME;
  });
});

describe("ConfigValidator", () => {
  const validator = new ConfigValidator();

  test("cross-field: test.seed without randomize", () => {
    const result = validator.validate({ test: { seed: 123 } }, "test.toml");
    expect(result.warnings.some(w => w.kind === "cross_field_warning")).toBe(true);
  });

  test("cross-field: run.shell system without server-first", () => {
    const result = validator.validate({ target: "local-first", run: { shell: "system" } }, "test.toml");
    expect(result.errors.some(e => e.kind === "cross_field_error")).toBe(true);
  });

  test("unknown key warning", () => {
    const result = validator.validate({ unknownKey: 123 }, "test.toml");
    expect(result.warnings.some(w => w.kind === "unknown_key")).toBe(true);
    expect(result.valid).toBe(true);
  });

  test("type checking with zod", () => {
      const result = validator.validate({ cryptomem: "not-a-boolean" }, "test.toml");
      expect(result.valid).toBe(false);
      expect(result.errors[0].kind).toBe("type_error");
      expect(result.errors[0].fieldPath).toBe("cryptomem");
  });

  test("line numbers in errors", () => {
      const content = `
cryptomem = "invalid"
[test]
root = 123
      `;
      const raw = { cryptomem: "invalid", test: { root: 123 } };
      const result = validator.validate(raw, "test.toml", content);
      expect(result.errors.find(e => e.fieldPath === "cryptomem")?.line).toBe(2);
      expect(result.errors.find(e => e.fieldPath === "test.root")?.line).toBe(4);
  });
});
