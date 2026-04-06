// src/core/plugins/config/config.test.ts
import { expect, test, describe, beforeEach } from "bun:test";
import { useConfig } from "./use_config";

describe("Plugin Config API", () => {
	// Mock globals
	(global as any).Alloy = { currentPluginName: "test-plugin" };
	(global as any).AlloyInternal = {
		__alloy_use_config: () => JSON.stringify({ dbPath: "./data/app.db", maxConnections: 10 }),
		__alloy_config_set: () => {},
	};

	test("useConfig returns accessor with initial values", () => {
		const config = useConfig({
			configs: {
				dbPath: { default: "./default.db" },
				maxConnections: { default: 5 },
			},
		});

		expect(config.dbPath).toBe("./data/app.db");
		expect(config.maxConnections).toBe(10);
	});

	test("config.set updates in-memory value", () => {
		const config = useConfig({
			configs: {
				dbPath: { default: "./default.db" },
			},
		});

		config.set("dbPath", "./new.db");
		expect(config.dbPath).toBe("./new.db");
	});
});
