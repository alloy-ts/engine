export type LogLevel = "debug" | "warn" | "error";
export type JsxMode = "react" | "react-jsx" | "react-jsxdev" | "preserve" | "solid";
export type LoaderName = "jsx" | "js" | "ts" | "tsx" | "css" | "file" | "json" | "toml" | "wasm" | "napi" | "base64" | "dataurl" | "text";
export type Target = "local-first" | "server-first";
export type AutoInstall = "auto" | "force" | "disable" | "fallback";
export type Linker = "isolated" | "hoisted";
export type RunShell = "system" | "alloy";
export type CoverageReporter = "text" | "lcov";

export interface CoverageThresholdObject {
	line?: number;
	function?: number;
	statement?: number;
}

export interface RegistryObject {
	url: string;
	token?: string;
}

export interface ScopeRegistryObject {
	url: string;
	token?: string;
	username?: string;
	password?: string;
}

export interface ConsoleConfig {
	depth?: number; // default: 2
}

export interface TestReporterConfig {
	dots?: boolean;
	junit?: string; // path to JUnit XML output file
}

export interface TestConfig {
	root?: string; // default: "."
	preload?: string[];
	pathIgnorePatterns?: string[];
	smol?: boolean;
	coverage?: boolean; // default: false
	coverageThreshold?: number | CoverageThresholdObject;
	coverageSkipTestFiles?: boolean; // default: false
	coveragePathIgnorePatterns?: string | string[];
	coverageReporter?: CoverageReporter[]; // default: ["text"]
	coverageDir?: string; // default: "coverage"
	randomize?: boolean; // default: false
	seed?: number;
	rerunEach?: number; // default: 1
	retry?: number; // default: 0
	concurrentTestGlob?: string;
	onlyFailures?: boolean; // default: false
	reporter?: TestReporterConfig;
}

export interface InstallCacheConfig {
	dir?: string;
	disable?: boolean;
	disableManifest?: boolean;
}

export interface InstallLockfileConfig {
	save?: boolean; // default: true
	print?: "yarn";
}

export interface InstallSecurityConfig {
	scanner?: string;
}

export interface InstallConfig {
	optional?: boolean; // default: true
	dev?: boolean; // default: true
	peer?: boolean; // default: true
	production?: boolean; // default: false
	exact?: boolean;
	saveTextLockfile?: boolean; // default: true
	auto?: AutoInstall;
	frozenLockfile?: boolean;
	dryRun?: boolean;
	globalDir?: string;
	globalBinDir?: string;
	registry?: string | RegistryObject;
	linkWorkspacePackages?: boolean; // default: true
	ca?: string;
	cafile?: string;
	cache?: InstallCacheConfig;
	lockfile?: InstallLockfileConfig;
	linker?: Linker; // default: "hoisted"
	security?: InstallSecurityConfig;
	minimumReleaseAge?: number;
	minimumReleaseAgeExcludes?: string[];
	scopes?: Record<string, string | ScopeRegistryObject>;
}

export interface RunConfig {
	shell?: RunShell;
	alloy?: boolean;
	silent?: boolean; // default: false
}

export interface DebugConfig {
	editor?: string;
}

export interface PluginFfiSymbol {
	args: string[]; // FFIType names
	returns: string; // FFIType name
}

export interface PluginFfiConfig {
	library: string;
	symbols: Record<string, PluginFfiSymbol>;
}

export interface PluginCapabilities {
	ffi?: boolean;
	browserAPIs?: string[];
	shellCapabilities?: string[];
}

export interface PluginManifest {
	name: string;
	version: string;
	description?: string;
	entry: string;
	dependencies?: string[];
	ffi?: PluginFfiConfig;
	capabilities?: PluginCapabilities;
}

export interface PluginEntry {
	name?: string;
	path?: string;
	config?: Record<string, unknown>;
	dependencies?: string[];
	disabled?: boolean;
}

export interface PluginConfigEntry {
	default: unknown;
}

export interface PluginConfigSchema {
	configs: Record<string, PluginConfigEntry>;
}

export interface AlloyConfig {
	// Runtime fields
	preload?: string[];
	jsx?: JsxMode;
	jsxFactory?: string;
	jsxFragment?: string;
	jsxImportSource?: string;
	cryptomem?: boolean;
	logLevel?: LogLevel; // default: "warn"
	define?: Record<string, string>;
	loader?: Record<string, LoaderName>;
	telemetry?: boolean; // default: true
	env?: boolean | { file: boolean };
	target?: Target; // default: "local-first"
	shellCapabilities?: string[]; // default: []
	shellCapabilityImages?: Record<string, string>;
	disableWasmEdge?: boolean; // default: false (local-first), true (server-first)
	useContainers?: boolean; // default: true (local-first), false (server-first)

	// Sections
	console?: ConsoleConfig;
	test?: TestConfig;
	install?: InstallConfig;
	run?: RunConfig;
	debug?: DebugConfig;
	plugins?: (string | PluginEntry)[];

	// Plugin-defined config sections — keyed by plugin name
	// Populated from [plugins.<name>] TOML sections
	pluginConfigs?: Record<string, Record<string, unknown>>;
}

export const BUILTIN_DEFAULTS_LOCAL: AlloyConfig = {
	target: "local-first",
	preload: [],
	cryptomem: false,
	logLevel: "warn",
	define: {},
	loader: {},
	telemetry: true,
	env: true,
	shellCapabilities: [],
	shellCapabilityImages: {},
	disableWasmEdge: false,
	useContainers: true,
	console: { depth: 2 },
	test: {
		root: ".",
		coverage: false,
		coverageSkipTestFiles: false,
		coverageReporter: ["text"],
		coverageDir: "coverage",
		randomize: false,
		rerunEach: 1,
		retry: 0,
		onlyFailures: false,
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

export const BUILTIN_DEFAULTS_SERVER: AlloyConfig = {
	target: "server-first",
	preload: [],
	cryptomem: false,
	logLevel: "warn",
	define: {},
	loader: {},
	telemetry: true,
	env: true,
	shellCapabilities: [],
	shellCapabilityImages: {},
	disableWasmEdge: true,
	useContainers: false,
	console: { depth: 2 },
	test: {
		root: ".",
		coverage: false,
		coverageSkipTestFiles: false,
		coverageReporter: ["text"],
		coverageDir: "coverage",
		randomize: false,
		rerunEach: 1,
		retry: 0,
		onlyFailures: false,
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
