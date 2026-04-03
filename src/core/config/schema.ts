import { z } from "zod";

const LogLevelSchema = z.enum(["debug", "warn", "error"]);
const JsxModeSchema = z.enum([
	"react",
	"react-jsx",
	"react-jsxdev",
	"preserve",
	"solid",
]);
const LoaderNameSchema = z.enum([
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
const TargetSchema = z.enum(["local-first", "server-first"]);
const AutoInstallSchema = z.enum(["auto", "force", "disable", "fallback"]);
const LinkerSchema = z.enum(["isolated", "hoisted"]);
const RunShellSchema = z.enum(["system", "alloy"]);
const CoverageReporterSchema = z.enum(["text", "lcov"]);

const CoverageThresholdObjectSchema = z.object({
	line: z.number().optional(),
	function: z.number().optional(),
	statement: z.number().optional(),
});

const RegistryObjectSchema = z.object({
	url: z.string(),
	token: z.string().optional(),
});

const ScopeRegistryObjectSchema = z.object({
	url: z.string(),
	token: z.string().optional(),
	username: z.string().optional(),
	password: z.string().optional(),
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
	plugins: z
		.array(
			z.union([
				z.string(),
				z.object({
					name: z.string().optional(),
					path: z.string().optional(),
					config: z.record(z.unknown()).optional(),
					dependencies: z.array(z.string()).optional(),
					disabled: z.boolean().optional(),
				}),
			]),
		)
		.optional(),

	console: z
		.object({
			depth: z.number().optional(),
		})
		.optional(),

	test: z
		.object({
			root: z.string().optional(),
			preload: z.array(z.string()).optional(),
			pathIgnorePatterns: z.array(z.string()).optional(),
			smol: z.boolean().optional(),
			coverage: z.boolean().optional(),
			coverageThreshold: z
				.union([z.number(), CoverageThresholdObjectSchema])
				.optional(),
			coverageSkipTestFiles: z.boolean().optional(),
			coveragePathIgnorePatterns: z.union([z.string(), z.array(z.string())]).optional(),
			coverageReporter: z.array(CoverageReporterSchema).optional(),
			coverageDir: z.string().optional(),
			randomize: z.boolean().optional(),
			seed: z.number().optional(),
			rerunEach: z.number().optional(),
			retry: z.number().optional(),
			concurrentTestGlob: z.string().optional(),
			onlyFailures: z.boolean().optional(),
			reporter: z
				.object({
					dots: z.boolean().optional(),
					junit: z.string().optional(),
				})
				.optional(),
		})
		.optional(),

	install: z
		.object({
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
			cache: z
				.object({
					dir: z.string().optional(),
					disable: z.boolean().optional(),
					disableManifest: z.boolean().optional(),
				})
				.optional(),
			lockfile: z
				.object({
					save: z.boolean().optional(),
					print: z.literal("yarn").optional(),
				})
				.optional(),
			linker: LinkerSchema.optional(),
			security: z
				.object({
					scanner: z.string().optional(),
				})
				.optional(),
			minimumReleaseAge: z.number().optional(),
			minimumReleaseAgeExcludes: z.array(z.string()).optional(),
			scopes: z.record(z.union([z.string(), ScopeRegistryObjectSchema])).optional(),
		})
		.optional(),

	run: z
		.object({
			shell: RunShellSchema.optional(),
			alloy: z.boolean().optional(),
			silent: z.boolean().optional(),
		})
		.optional(),

	debug: z
		.object({
			editor: z.string().optional(),
		})
		.optional(),
});
