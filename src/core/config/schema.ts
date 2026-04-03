export interface AlloyConfig {
	preload?: string[];
	jsx?: "react" | "react-jsx" | "react-jsxdev" | "preserve" | "solid";
	jsxFactory?: string;
	jsxFragment?: string;
	jsxImportSource?: string;
	cryptomem?: boolean;
	logLevel?: "debug" | "warn" | "error";
	define?: Record<string, string>;
	loader?: Record<string, "jsx" | "js" | "ts" | "tsx" | "css" | "file" | "json" | "toml" | "wasm" | "napi" | "base64" | "dataurl" | "text">;
	telemetry?: boolean;
	env?: boolean | { file: boolean };
	console?: { depth: number };
	test?: TestConfig;
	install?: InstallConfig;
	run?: RunConfig;
	debug?: DebugConfig;
	target?: "local-first" | "server-first";
	shellCapabilities?: string[];
	disableWasmEdge?: boolean;
	useContainers?: boolean;
	shellCapabilityImages?: Record<string, string>;
}

export interface TestConfig {
	root?: string;
	preload?: string[];
	pathIgnorePatterns?: string[];
	smol?: boolean;
	coverage?: boolean;
	coverageThreshold?: number | { line?: number; function?: number; statement?: number };
	coverageSkipTestFiles?: boolean;
	coveragePathIgnorePatterns?: string[];
	coverageReporter?: ("text" | "lcov")[];
	coverageDir?: string;
	randomize?: boolean;
	seed?: number;
	rerunEach?: number;
	retry?: number;
	concurrentTestGlob?: string;
	onlyFailures?: boolean;
	reporter?: { dots?: boolean; junit?: string };
}

export interface InstallConfig {
	optional?: boolean;
	dev?: boolean;
	peer?: boolean;
	production?: boolean;
	exact?: boolean;
	saveTextLockfile?: boolean;
	auto?: "auto" | "force" | "disable" | "fallback";
	frozenLockfile?: boolean;
	dryRun?: boolean;
	globalDir?: string;
	globalBinDir?: string;
	registry?: string | { url: string; token: string };
	linkWorkspacePackages?: boolean;
	scopes?: Record<string, string | { url: string; token?: string; username?: string; password?: string }>;
	ca?: string;
	cafile?: string;
	cache?: { dir?: string; disable?: boolean; disableManifest?: boolean };
	lockfile?: { save?: boolean; print?: "yarn" };
	linker?: "isolated" | "hoisted";
	security?: { scanner?: string };
	minimumReleaseAge?: number;
	minimumReleaseAgeExcludes?: string[];
}

export interface RunConfig {
	shell?: "system" | "alloy";
	alloy?: boolean;
	silent?: boolean;
}

export interface DebugConfig {
	editor?: "code" | "subl" | "vim" | "webstorm" | "idea" | string;
}

export const DEFAULT_CONFIG: AlloyConfig = {
	telemetry: true,
	env: true,
	console: { depth: 2 },
	target: "local-first",
	test: {
		root: ".",
		coverage: false,
		coverageSkipTestFiles: false,
		coverageReporter: ["text"],
		coverageDir: "coverage",
		randomize: false,
		rerunEach: 1,
		retry: 0,
	},
	install: {
		optional: true,
		dev: true,
		peer: true,
		production: false,
		saveTextLockfile: true,
		linkWorkspacePackages: true,
		lockfile: { save: true },
		linker: "hoisted",
	},
	run: {
		silent: false,
	},
};
