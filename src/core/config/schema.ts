import { z } from "zod";

export const AlloyConfigSchema = z.object({
    target: z.enum(["local-first", "server-first"]).default("local-first"),
    logLevel: z.enum(["debug", "warn", "error"]).default("warn"),
    define: z.record(z.string()).default({}),
    shellCapabilities: z.array(z.string()).default([]),
    useContainers: z.boolean().optional(),
    disableWasmEdge: z.boolean().optional(),
    pluginConfigs: z.record(z.any()).default({}),
    plugins: z.array(z.union([z.string(), z.any()])).default([])
});

export type AlloyConfig = z.infer<typeof AlloyConfigSchema>;
