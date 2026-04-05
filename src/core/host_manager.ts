import { existsSync, mkdirSync, writeFileSync, chmodSync } from "node:fs";
import { join } from "node:path";
import { homedir } from "node:os";

export async function ensureAlloyHost(): Promise<string> {
    const platform = process.platform;
    const arch = process.arch;
    const cacheDir = join(homedir(), ".alloy", "bin", `${platform}-${arch}`);
    const ext = platform === "win32" ? ".exe" : "";
    const binaryPath = join(cacheDir, "alloy_host" + ext);

    if (existsSync(binaryPath)) {
        return binaryPath;
    }

    // In a real implementation, we would download the binary from GitHub releases.
    // For this environment, we assume the binary might need to be built or is already present in a known location.
    // As a fallback for the simulation, we'll look in the current build directory.
    const localBuildPath = join(process.cwd(), "src", "core", "test_engine" + ext);
    if (existsSync(localBuildPath)) {
        if (!existsSync(cacheDir)) {
            mkdirSync(cacheDir, { recursive: true });
        }
        const data = await Bun.file(localBuildPath).arrayBuffer();
        await Bun.write(binaryPath, data);
        chmodSync(binaryPath, 0o755);
        return binaryPath;
    }

    throw new Error(`AlloyScript host binary not found for ${platform}-${arch}. Please run 'make' in src/core.`);
}
