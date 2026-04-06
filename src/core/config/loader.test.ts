import { test, expect } from "bun:test";
import { ConfigLoader } from "./loader";
import { AlloyConfig } from "./config";

test("Merge Precedence", () => {
	const loader = new ConfigLoader();
	const user: Partial<AlloyConfig> = {
		logLevel: "debug",
		define: { A: "1", B: "2" },
	};
	const workspace: Partial<AlloyConfig> = {
		logLevel: "warn",
		define: { B: "3", C: "4" },
	};
	const cliFlags: Partial<AlloyConfig> = {
		logLevel: "error",
	};

	const merged = (loader as any).merge(user, workspace, cliFlags);
	expect(merged.logLevel).toBe("error");
	expect(merged.define.A).toBe("1");
	expect(merged.define.B).toBe("3");
	expect(merged.define.C).toBe("4");
});

test("Env Var Interpolation", () => {
	const loader = new ConfigLoader();
	process.env.FOO = "bar";
	const config = {
		test: {
			root: "$FOO/tests",
			coverageDir: "${FOO}_coverage",
		},
	};
	const interpolated = (loader as any).interpolate(config);
	expect(interpolated.test.root).toBe("bar/tests");
	expect(interpolated.test.coverageDir).toBe("bar_coverage");
});
