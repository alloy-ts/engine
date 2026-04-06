export type LogLevel = "debug" | "warn" | "error";
export type JsxMode =
	| "react"
	| "react-jsx"
	| "react-jsxdev"
	| "preserve"
	| "solid";
export type LoaderName =
	| "jsx"
	| "js"
	| "ts"
	| "tsx"
	| "css"
	| "file"
	| "json"
	| "toml"
	| "wasm"
	| "napi"
	| "base64"
	| "dataurl"
	| "text";
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
	depth?: number;
}

export interface TestReporterConfig {
	dots?: boolean;
	junit?: string;
}

export interface TestConfig {
	root?: string;
	preload?: string[];
	pathIgnorePatterns?: string[];
	smol?: boolean;
	coverage?: boolean;
	coverageThreshold?: number | CoverageThresholdObject;
	coverageSkipTestFiles?: boolean;
	coveragePathIgnorePatterns?: string | string[];
	coverageReporter?: CoverageReporter[];
	coverageDir?: string;
	randomize?: boolean;
	seed?: number;
	rerunEach?: number;
	retry?: number;
	concurrentTestGlob?: string;
	onlyFailures?: boolean;
	reporter?: TestReporterConfig;
}

export interface InstallCacheConfig {
	dir?: string;
	disable?: boolean;
	disableManifest?: boolean;
}

export interface InstallLockfileConfig {
	save?: boolean;
	print?: "yarn";
}

export interface InstallSecurityConfig {
	scanner?: string;
}

export interface InstallConfig {
	optional?: boolean;
	dev?: boolean;
	peer?: boolean;
	production?: boolean;
	exact?: boolean;
	saveTextLockfile?: boolean;
	auto?: AutoInstall;
	frozenLockfile?: boolean;
	dryRun?: boolean;
	globalDir?: string;
	globalBinDir?: string;
	registry?: string | RegistryObject;
	linkWorkspacePackages?: boolean;
	ca?: string;
	cafile?: string;
	cache?: InstallCacheConfig;
	lockfile?: InstallLockfileConfig;
	linker?: Linker;
	minimumReleaseAge?: number;
	minimumReleaseAgeExcludes?: string[];
	scopes?: Record<string, string | ScopeRegistryObject>;
}

export interface RunConfig {
	shell?: RunShell;
	alloy?: boolean;
	silent?: boolean;
}

export interface DebugConfig {
	editor?: string;
}

export interface PluginFfiSymbol {
	args: string[];
	returns: string;
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
	pluginName: string;
	configs: Record<string, PluginConfigEntry>;
}

export interface AlloyConfig {
	preload?: string[];
	jsx?: JsxMode;
	jsxFactory?: string;
	jsxFragment?: string;
	jsxImportSource?: string;
	cryptomem?: boolean;
	logLevel?: LogLevel;
	define?: Record<string, string>;
	loader?: Record<string, LoaderName>;
	telemetry?: boolean;
	env?: boolean | { file: boolean };
	target?: Target;
	shellCapabilities?: string[];
	shellCapabilityImages?: Record<string, string>;
	disableWasmEdge?: boolean;
	useContainers?: boolean;
	/** Disable the alloy:web sidecar entirely. No fetch, crypto, URL, etc.
	 *  Results in a smaller binary with no WebView dependency. */
	noWebAPI?: boolean;
	console?: ConsoleConfig;
	test?: TestConfig;
	install?: InstallConfig;
	run?: RunConfig;
	debug?: DebugConfig;
	plugins?: (string | PluginEntry)[];
	pluginConfigs?: Record<string, Record<string, unknown>>;
}

export const BUILTIN_DEFAULTS: AlloyConfig = {
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
	noWebAPI: false,
	console: {
		depth: 2,
	},
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
		lockfile: {
			save: true,
		},
		linker: "hoisted",
	},
	run: {
		silent: false,
	},
};
