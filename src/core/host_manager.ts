import { BinaryManager } from "./engine";

/**
 * Ensures the alloy_host binary is present and up to date.
 */
export async function ensureAlloyHost(version: string = "0.1.0"): Promise<string> {
    const manager = new BinaryManager();
    return await manager.ensureBinary(version);
}
