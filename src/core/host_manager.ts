import { BinaryManager } from "./engine";

/**
 * Ensures the native alloy_host binary is available and verified.
 */
export async function ensureAlloyHost(version: string = "0.1.0"): Promise<string> {
    const manager = new BinaryManager();
    return await manager.ensureBinary(version);
}
