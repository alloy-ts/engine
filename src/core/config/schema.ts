// src/core/config/schema.ts
import { z } from "zod";

export const LogLevelSchema = z.enum(["debug", "warn", "error"]);
export const JsxModeSchema = z.enum([
	"react",
	"react-jsx",
	"react-jsxdev",
	"preserve",
	"solid",
]);
export const LoaderNameSchema = z.enum([
	"jsx",
	"js",
	"ts",
	"tsx",
	"css",
	"file",
	"json",
	"toml",
	"wasm",
	"napi",
	"base64",
	"dataurl",
	"text",
]);
export const TargetSchema = z.enum(["local-first", "server-first"]);
export const AutoInstallSchema = z.enum([
	"auto",
	"force",
	"disable",
	"fallback",
]);
export const LinkerSchema = z.enum(["isolated", "hoisted"]);
export const RunShellSchema = z.enum(["system", "alloy"]);
export const CoverageReporterSchema = z.enum(["text", "lcov"]);

export const CoverageThresholdObjectSchema = z.object({
	line: z.number().optional(),
	function: z.number().optional(),
	statement: z.number().optional(),
});

export const RegistryObjectSchema = z.object({
	url: z.string(),
	token: z.string().optional(),
});

export const ScopeRegistryObjectSchema = z.object({
	url: z.string(),
	token: z.string().optional(),
	username: z.string().optional(),
	password: z.string().optional(),
});

export const ConsoleConfigSchema = z.object({
	depth: z.number().optional(),
});

export const TestReporterConfigSchema = z.object({
	dots: z.boolean().optional(),
	junit: z.string().optional(),
});

export const TestConfigSchema = z.object({
	root: z.string().optional(),
	preload: z.array(z.string()).optional(),
	pathIgnorePatterns: z.array(z.string()).optional(),
	smol: z.boolean().optional(),
	coverage: z.boolean().optional(),
	coverageThreshold: z
		.union([z.number(), CoverageThresholdObjectSchema])
		.optional(),
	coverageSkipTestFiles: z.boolean().optional(),
	coveragePathIgnorePatterns: z
		.union([z.string(), z.array(z.string())])
		.optional(),
	coverageReporter: z.array(CoverageReporterSchema).optional(),
	coverageDir: z.string().optional(),
	randomize: z.boolean().optional(),
	seed: z.number().optional(),
	rerunEach: z.number().optional(),
	retry: z.number().optional(),
	concurrentTestGlob: z.string().optional(),
	onlyFailures: z.boolean().optional(),
	reporter: TestReporterConfigSchema.optional(),
});

export const InstallCacheConfigSchema = z.object({
	dir: z.string().optional(),
	disable: z.boolean().optional(),
	disableManifest: z.boolean().optional(),
});

export const InstallLockfileConfigSchema = z.object({
	save: z.boolean().optional(),
	print: z.enum(["yarn"]).optional(),
});

export const InstallSecurityConfigSchema = z.object({
	scanner: z.string().optional(),
});

export const InstallConfigSchema = z.object({
	optional: z.boolean().optional(),
	dev: z.boolean().optional(),
	peer: z.boolean().optional(),
	production: z.boolean().optional(),
	exact: z.boolean().optional(),
	saveTextLockfile: z.boolean().optional(),
	auto: AutoInstallSchema.optional(),
	frozenLockfile: z.boolean().optional(),
	dryRun: z.boolean().optional(),
	globalDir: z.string().optional(),
	globalBinDir: z.string().optional(),
	registry: z.union([z.string(), RegistryObjectSchema]).optional(),
	linkWorkspacePackages: z.boolean().optional(),
	ca: z.string().optional(),
	cafile: z.string().optional(),
	cache: InstallCacheConfigSchema.optional(),
	lockfile: InstallLockfileConfigSchema.optional(),
	linker: LinkerSchema.optional(),
	security: InstallSecurityConfigSchema.optional(),
	minimumReleaseAge: z.number().optional(),
	minimumReleaseAgeExcludes: z.array(z.string()).optional(),
	scopes: z.record(z.union([z.string(), ScopeRegistryObjectSchema])).optional(),
});

export const RunConfigSchema = z.object({
	shell: RunShellSchema.optional(),
	alloy: z.boolean().optional(),
	silent: z.boolean().optional(),
});

export const DebugConfigSchema = z.object({
	editor: z.string().optional(),
});

export const PluginFfiSymbolSchema = z.object({
	args: z.array(z.string()),
	returns: z.string(),
});

export const PluginFfiConfigSchema = z.object({
	library: z.string(),
	symbols: z.record(PluginFfiSymbolSchema),
});

export const PluginCapabilitiesSchema = z.object({
	ffi: z.boolean().optional(),
	browserAPIs: z.array(z.string()).optional(),
	shellCapabilities: z.array(z.string()).optional(),
});

export const PluginManifestSchema = z.object({
	name: z.string(),
	version: z.string(),
	description: z.string().optional(),
	entry: z.string(),
	dependencies: z.array(z.string()).optional(),
	ffi: PluginFfiConfigSchema.optional(),
	capabilities: PluginCapabilitiesSchema.optional(),
});

export const PluginEntrySchema = z.object({
	name: z.string().optional(),
	path: z.string().optional(),
	config: z.record(z.unknown()).optional(),
	dependencies: z.array(z.string()).optional(),
	disabled: z.boolean().optional(),
});

export const AlloyConfigSchema = z.object({
	preload: z.array(z.string()).optional(),
	jsx: JsxModeSchema.optional(),
	jsxFactory: z.string().optional(),
	jsxFragment: z.string().optional(),
	jsxImportSource: z.string().optional(),
	cryptomem: z.boolean().optional(),
	logLevel: LogLevelSchema.optional(),
	define: z.record(z.string()).optional(),
	loader: z.record(LoaderNameSchema).optional(),
	telemetry: z.boolean().optional(),
	env: z.union([z.boolean(), z.object({ file: z.boolean() })]).optional(),
	target: TargetSchema.optional(),
	shellCapabilities: z.array(z.string()).optional(),
	shellCapabilityImages: z.record(z.string()).optional(),
	disableWasmEdge: z.boolean().optional(),
	useContainers: z.boolean().optional(),

	console: ConsoleConfigSchema.optional(),
	test: TestConfigSchema.optional(),
	install: InstallConfigSchema.optional(),
	run: RunConfigSchema.optional(),
	debug: DebugConfigSchema.optional(),
	plugins: z.array(z.union([z.string(), PluginEntrySchema])).optional(),
});
