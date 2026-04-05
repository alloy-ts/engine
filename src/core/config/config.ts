export type LogLevel = "debug" | "warn" | "error";
export type Target = "local-first" | "server-first";

export interface TestConfig {
	root?: string;
	coverage?: boolean;
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

export interface AlloyConfig {
	target?: Target;
	logLevel?: LogLevel;
	shellCapabilities?: string[];
	shellCapabilityImages?: Record<string, string>;
	disableWasmEdge?: boolean;
	useContainers?: boolean;
	test?: TestConfig;
	define?: Record<string, string>;
	plugins?: (string | PluginEntry)[];
}

export const DEFAULTS_LOCAL: AlloyConfig = {
	target: "local-first",
	logLevel: "warn",
	shellCapabilities: [],
	shellCapabilityImages: {},
	disableWasmEdge: false,
	useContainers: true,
	test: { root: ".", coverage: false },
};

export const DEFAULTS_SERVER: AlloyConfig = {
	target: "server-first",
	logLevel: "warn",
	shellCapabilities: [],
	shellCapabilityImages: {},
	disableWasmEdge: true,
	useContainers: false,
	test: { root: ".", coverage: false },
};
