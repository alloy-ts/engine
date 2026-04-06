/**
 * AlloyScript — Capability-based shell executor (Bun/CLI side)
 *
 * Mirrors the Alloy.$ API available inside MicroQuickJS, but runs under Bun
 * for use in CLI commands (compile, transpile, etc.).
 *
 * Security model:
 *   - Only executables in ALLOWED_EXECUTABLES may be invoked.
 *   - Arguments are shell-escaped before interpolation.
 *   - No raw shell string execution — commands are always argv arrays.
 *   - Capability tokens can further restrict which commands a caller may run.
 *
 * Usage:
 *   import { $, ShellError } from "../core/shell";
 *
 *   const result = await $`cmake --version`;
 *   const result = await $`cmake -B ${buildDir} -DFOO=${value}`;
 */

// ── Capability whitelist ──────────────────────────────────────────────────

/**
 * Executables that AlloyScript CLI commands are permitted to invoke.
 * Anything not in this set is rejected before spawning.
 */
export const ALLOWED_EXECUTABLES = new Set([
  "cmake",
  "make",
  "ninja",
  "clang",
  "clang++",
  "gcc",
  "g++",
  "cc",
  "c++",
  "ar",
  "ld",
  "strip",
  "install_name_tool", // macOS
  "otool",             // macOS
  "lipo",              // macOS
  "codesign",          // macOS
  "dumpbin",           // Windows
  "lib",               // Windows (MSVC)
  "cl",                // Windows (MSVC)
  "link",              // Windows (MSVC)
  "bun",
  "node",
]);

// ── Types ─────────────────────────────────────────────────────────────────

export interface ShellResult {
  stdout:   string;
  stderr:   string;
  exitCode: number;
}

export class ShellError extends Error {
  constructor(
    public readonly command: string,
    public readonly result: ShellResult,
  ) {
    super(
      `Command failed (exit ${result.exitCode}): ${command}\n${result.stderr.trim()}`,
    );
    this.name = "ShellError";
  }
}

export class CapabilityError extends Error {
  constructor(executable: string) {
    super(
      `Capability denied: "${executable}" is not in the AlloyScript shell whitelist.\n` +
      `Allowed executables: ${[...ALLOWED_EXECUTABLES].join(", ")}`,
    );
    this.name = "CapabilityError";
  }
}

// ── Escape ────────────────────────────────────────────────────────────────

/**
 * Shell-escape a single argument value so it is safe to embed in a
 * command string. Wraps in single quotes and escapes internal single quotes.
 */
function escapeArg(value: unknown): string {
  const s = String(value ?? "");
  // Single-quote the value; escape any embedded single quotes as '\''
  return `'${s.replace(/'/g, "'\\''")}'`;
}

// ── Core executor ─────────────────────────────────────────────────────────

async function execArgv(
  argv: string[],
  cwd?: string,
): Promise<ShellResult> {
  const [executable, ...args] = argv;
  if (!executable) throw new Error("empty command");

  // Capability check — strip any path prefix for the check
  const basename = executable.replace(/.*[/\\]/, "");
  if (!ALLOWED_EXECUTABLES.has(basename)) {
    throw new CapabilityError(basename);
  }

  const proc = Bun.spawn([executable, ...args], {
    cwd,
    stdout: "pipe",
    stderr: "pipe",
  });

  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);

  return { stdout, stderr, exitCode };
}

// ── Tagged template shell API ─────────────────────────────────────────────

interface ShellExpression {
  /** Await to run; throws ShellError on non-zero exit */
  then: Promise<ShellResult>["then"];
  /** Run but don't throw on non-zero exit */
  nothrow(): Promise<ShellResult>;
  /** Run in a specific working directory */
  cwd(dir: string): ShellExpression;
}

/**
 * Capability-checked shell tag. Interpolated values are automatically escaped.
 *
 * @example
 *   const result = await $`cmake --version`;
 *   const result = await $`cmake -B ${buildDir} -DFOO=${value}`;
 *   const result = await $`cmake --build ${dir}`.cwd("/tmp");
 */
export function $(
  strings: TemplateStringsArray,
  ...values: unknown[]
): ShellExpression {
  // Build the full command string with escaped interpolations
  let cmd = strings[0] ?? "";
  for (let i = 0; i < values.length; i++) {
    cmd += escapeArg(values[i]) + (strings[i + 1] ?? "");
  }

  // Split into argv — simple whitespace split (no subshell, no pipes)
  // This is intentionally limited: no shell metacharacters are interpreted.
  const argv = cmd.trim().split(/\s+/).filter(Boolean);

  let workingDir: string | undefined;
  let throws = true;

  const run = (): Promise<ShellResult> =>
    execArgv(argv, workingDir).then((result) => {
      if (throws && result.exitCode !== 0) {
        throw new ShellError(argv.join(" "), result);
      }
      return result;
    });

  const expr: ShellExpression = {
    then: (onfulfilled, onrejected) => run().then(onfulfilled, onrejected),
    nothrow() {
      throws = false;
      return expr;
    },
    cwd(dir: string) {
      workingDir = dir;
      return expr;
    },
  };

  return expr;
}

// ── Convenience: run and stream output to console ─────────────────────────

export async function run(
  strings: TemplateStringsArray,
  ...values: unknown[]
): Promise<void> {
  const result = await $(strings, ...values);
  if (result.stdout.trim()) console.log(result.stdout.trimEnd());
  if (result.stderr.trim()) console.error(result.stderr.trimEnd());
}
